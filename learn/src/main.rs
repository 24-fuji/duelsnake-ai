mod config;
mod env;
mod export;
mod model;
mod replay;
mod trainer;

use clap::Parser;
use config::Config;
use env::observation::{GRID_CHANNELS, VECTOR_FEATURES};
use export::ModelFile;
use replay::ReplayBuffer;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tch::Device;
use trainer::{OutputPaths, Trainer};

const CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// DuelSnake-AI の自己対戦強化学習。Ctrl+C で中断すると、その時点のモデルを JSON で保存して終了する
#[derive(Parser)]
struct Cli {
    /// 設定ファイル [既定: learn/config.yaml]
    #[arg(long)]
    config: Option<PathBuf>,
    /// モデルの出力先 [既定: model/]
    #[arg(long)]
    out_dir: Option<PathBuf>,
    /// 保存済みモデル JSON から学習を再開する [値を省略すると <out_dir>/recent-model/snake-model.json]
    #[arg(long, value_name = "MODEL_JSON")]
    resume: Option<Option<PathBuf>>,
    /// この回数の対戦を終えたら終了する [既定: Ctrl+C まで続ける]
    #[arg(long)]
    games: Option<u64>,
    /// 同時に進める対戦数 (config の train.num_envs を上書き)
    #[arg(long)]
    envs: Option<usize>,
    /// libtorch のスレッド数 [既定: 論理コア数の半分]
    /// 小さなネットワークでは論理コアをすべて使うとかえって遅くなる
    #[arg(long)]
    threads: Option<usize>,
    /// 乱数シード [既定: ランダム]
    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let crate_dir = Path::new(CRATE_DIR);
    let config_path = cli.config.unwrap_or_else(|| crate_dir.join("config.yaml"));
    let out_dir = cli.out_dir.unwrap_or_else(|| crate_dir.join("../model"));
    let recent_model = out_dir.join("recent-model/snake-model.json");

    let mut config = Config::load(&config_path)?;
    if let Some(envs) = cli.envs {
        config.train.num_envs = envs.max(1);
    }
    let threads = cli.threads.unwrap_or_else(|| {
        let logical = std::thread::available_parallelism().map_or(1, |n| n.get());
        (logical / 2).max(1)
    });
    tch::set_num_threads(threads as i32);
    let device = Device::cuda_if_available();
    let seed = cli.seed.unwrap_or_else(rand::random);

    let resume = match cli.resume {
        Some(path) => {
            let path = path.unwrap_or_else(|| recent_model.clone());
            println!("学習を再開します: {}", path.display());
            Some(ModelFile::read(&path)?)
        }
        None => None,
    };

    let mut trainer = Trainer::new(config.clone(), device, seed, resume)?;
    let grid_len = GRID_CHANNELS * (config.game.grid.width * config.game.grid.height) as usize;
    let replay_mb = (config.train.replay_capacity
        * ReplayBuffer::bytes_per_transition(grid_len, VECTOR_FEATURES)) as f64
        / 1e6;
    let resumed = trainer.info();
    println!("DuelSnake-AI 自己対戦学習");
    println!("  設定       : {}", config_path.display());
    println!(
        "  デバイス   : {device:?} / スレッド {threads} / 同時対戦 {}",
        config.train.num_envs
    );
    println!(
        "  ネットワーク: {} パラメータ / リプレイ最大 {replay_mb:.0} MB",
        trainer.spec().parameter_count()
    );
    println!("  シード     : {seed}");
    if resumed.games > 0 {
        println!(
            "  再開時点   : {} 試合 / {} 判断 / {} 更新",
            resumed.games, resumed.decisions, resumed.updates
        );
    }
    println!("Ctrl+C で中断するとモデルを保存して終了します");

    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    ctrlc::set_handler(move || {
        if flag.swap(false, Ordering::SeqCst) {
            println!(
                "\n[SIGINT] 中断します。モデルを保存しています... (もう一度押すと保存せず終了)"
            );
        } else {
            std::process::exit(130);
        }
    })?;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let paths = OutputPaths {
        recent_model,
        backup_dir: out_dir.join("models"),
        log_file: crate_dir.join(format!("../log/train-{timestamp}.csv")),
    };
    trainer.run(&running, cli.games, &paths)?;
    println!("学習を終了しました");
    Ok(())
}
