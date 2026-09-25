mod config;
mod env;
mod export;
mod force;
mod model;
mod monitor;
mod replay;
mod trainer;

use clap::Parser;
use config::Config;
use env::observation::{GRID_CHANNELS, VECTOR_FEATURES};
use export::ModelFile;
use monitor::TrainLog;
use replay::ReplayBuffer;
use std::fs;
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
    /// 保存済みモデル JSON から学習を再開する。値を省略すると config.yaml の盤面サイズのモデル
    /// (<out_dir>/recent-model/snake-model-<幅>x<高さ>.json) から再開し、無ければ警告を出して新しく学習する
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
    let repo_dir = crate_dir.parent().unwrap_or(crate_dir);
    let config_path = cli.config.unwrap_or_else(|| crate_dir.join("config.yaml"));
    let out_dir = cli.out_dir.unwrap_or_else(|| repo_dir.join("model"));

    let mut config = Config::load(&config_path)?;
    let size = export::size_label(&config.game.grid);
    let recent_dir = out_dir.join("recent-model");
    let recent_model = recent_dir.join(export::model_file_name(&config.game.grid));
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

    let mut notices = Vec::new();
    let resume = match cli.resume {
        Some(Some(path)) => Some(read_resume_model(&path, &config, &mut notices)?),
        Some(None) if recent_model.exists() => {
            Some(read_resume_model(&recent_model, &config, &mut notices)?)
        }
        Some(None) => {
            let mut warning = format!(
                "警告: 盤面 {size} のモデル {} が無いので、新しく学習を始めます",
                recent_model.display()
            );
            let others = model_sizes_in(&recent_dir);
            if !others.is_empty() {
                warning += &format!(
                    " (今ある盤面: {})。別の盤面で続けるなら config.yaml の game.grid を変えてください",
                    others.join(", ")
                );
            }
            notices.push(warning);
            None
        }
        None => None,
    };

    let mut trainer = Trainer::new(config.clone(), device, seed, resume)?;
    let grid_len = GRID_CHANNELS * (config.game.grid.width * config.game.grid.height) as usize;
    let replay_mb = (config.train.replay_capacity
        * ReplayBuffer::bytes_per_transition(grid_len, VECTOR_FEATURES)) as f64
        / 1e6;
    let resumed = trainer.info();
    let force = &config.reward.force;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let mut log = TrainLog::create(&repo_dir.join(format!("log/train-{timestamp}")))?;
    let mut lines = vec![
        "DuelSnake-AI 自己対戦学習".to_string(),
        format!("  設定       : {}", config_path.display()),
        format!("  盤面       : {size}"),
        format!(
            "  デバイス   : {device:?} / スレッド {threads} / 同時対戦 {}",
            config.train.num_envs
        ),
        format!(
            "  ネットワーク: {} パラメータ / リプレイ最大 {replay_mb:.0} MB",
            trainer.spec().parameter_count()
        ),
        format!("  シード     : {seed}"),
        format!(
            "  引力・斥力 : リンゴ {} / 金のリンゴ {} / 毒リンゴ {} (range {})",
            force.normal_apple_attraction,
            force.gold_apple_attraction,
            force.poison_apple_repulsion,
            force.range
        ),
    ];
    if resumed.games > 0 {
        lines.push(format!(
            "  再開時点   : {} 試合 / {} 判断 / {} 更新",
            resumed.games, resumed.decisions, resumed.updates
        ));
    }
    lines.extend(notices);
    for line in &lines {
        println!("{line}");
        log.event(line.trim_start())?;
    }
    println!(
        "  詳しいログ : {} (別の端末で `just train-watch` を実行すると追えます)",
        log.text_path().display()
    );
    println!("Ctrl+C で中断するとモデルを保存して終了します");

    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    ctrlc::set_handler(move || {
        // 1回目は学習ループに止まってもらい、保存してから終了する。2回目は保存せずに終了する
        if !flag.swap(false, Ordering::SeqCst) {
            println!();
            std::process::exit(130);
        }
    })?;

    let paths = OutputPaths {
        recent_model,
        backup_dir: out_dir.join("models"),
    };
    trainer.run(&running, cli.games, &paths, &mut log)?;
    println!("学習を終了しました");
    log.event("学習を終了しました")?;
    Ok(())
}

/// 再開元のモデルを読む。盤面サイズが config.yaml と違えばエラーにする
fn read_resume_model(
    path: &Path,
    config: &Config,
    notices: &mut Vec<String>,
) -> Result<ModelFile, Box<dyn std::error::Error>> {
    let model = ModelFile::read(path)?;
    let (model_size, size) = (
        export::size_label(&model.game.grid),
        export::size_label(&config.game.grid),
    );
    if model_size != size {
        return Err(format!(
            "再開元モデル {} の盤面 {model_size} が config.yaml の盤面 {size} と異なります",
            path.display()
        )
        .into());
    }
    notices.push(format!("学習を再開します: {}", path.display()));
    if model.game != config.game {
        notices.push("注意: 再開元モデルと config.yaml のゲームルールが異なります。現在の config.yaml のルールで学習を続けます".to_string());
    }
    Ok(model)
}

/// `dir` にある盤面サイズごとのモデルの盤面サイズ
fn model_sizes_in(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut sizes: Vec<String> = entries
        .filter_map(|e| {
            let name = e.ok()?.file_name();
            export::size_from_file_name(name.to_str()?).map(str::to_string)
        })
        .collect();
    sizes.sort();
    sizes
}
