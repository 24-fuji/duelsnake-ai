mod arena;
mod config;
mod env;
mod export;
mod force;
mod git;
mod model;
mod monitor;
mod progress;
mod replay;
mod trainer;
mod trap;

use arena::{Contender, CONTENDERS, NEW, OLD};
use clap::{Args, Parser, Subcommand};
use config::Config;
use env::observation::{GRID_CHANNELS, VECTOR_FEATURES};
use env::rules::Rules;
use export::ModelFile;
use monitor::{thousands, TrainLog};
use progress::ProgressFiles;
use replay::ReplayBuffer;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tch::Device;
use trainer::{CommitPolicy, EvalTarget, OutputPaths, Trainer};

const CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");
/// 盤面サイズごとの最新モデルを置くフォルダ (<out_dir> の下)。Web アプリはここのモデルを使う
const RECENT_DIR: &str = "recent-model";
/// 学習の経過 (旧モデルとの対戦の勝率の CSV とグラフ) を置くフォルダ (リポジトリ直下から)
const PROGRESS_DIR: &str = "docs/training";
/// 旧モデルを置くフォルダ (<out_dir> の下)。新モデルとの対戦に使う
const PREVIOUS_DIR: &str = "previous-model";
/// 旧モデルを取り出すブランチと、そのブランチでの最新モデルのフォルダ (リポジトリ直下から)
const PREVIOUS_MODEL_BRANCH: &str = "main";
const BRANCH_MODEL_DIR: &str = "model/recent-model";

/// DuelSnake-AI の自己対戦強化学習。Ctrl+C で中断すると、その時点のモデルを JSON で保存して終了する。
/// 学習を始める前に、main ブランチにある同じ盤面のモデルを <out_dir>/previous-model/ に旧モデルとして置く
#[derive(Parser)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
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
    /// 通算試合数が config の train.commit_interval_games の倍数を越えるごとに、
    /// 最新モデルを "model update" で git にコミットしてプッシュする
    #[arg(long)]
    auto_commit: bool,
    /// Ctrl+C で中断したとき、保存した最新モデルを "model update" で git にコミットしてプッシュする
    #[arg(long)]
    commit_on_interrupt: bool,
}

#[derive(Subcommand)]
enum Command {
    /// 旧モデルと新モデルを対戦させて強さを比べる (学習はしない)
    Versus(VersusArgs),
}

#[derive(Args)]
struct VersusArgs {
    /// この盤面だけで対戦する (例: 8-8 は幅 8・高さ 8。8x8 や、正方形なら 8 だけでもよい)
    /// [既定: main と <out_dir>/recent-model/ の両方にモデルがある、すべての盤面で対戦する]
    #[arg(long, value_name = "W-H", value_parser = parse_size)]
    size: Option<(i32, i32)>,
    /// 旧モデル [既定: <out_dir>/previous-model/snake-model-<幅>x<高さ>.json。対戦の前に main ブランチのモデルで置き換える]
    /// 指定すると、--size が無ければこのモデルの盤面だけで対戦する
    #[arg(long, value_name = "MODEL_JSON")]
    old: Option<PathBuf>,
    /// 新モデル [既定: <out_dir>/recent-model/snake-model-<幅>x<高さ>.json]
    /// 指定すると、--size が無ければこのモデルの盤面だけで対戦する
    #[arg(long, value_name = "MODEL_JSON")]
    new: Option<PathBuf>,
    /// 1つの盤面での対戦数。同じ盤面で先手と後手を入れ替えて 2 試合ずつ行うので偶数がよい
    #[arg(long, default_value_t = 1000)]
    games: u64,
    /// 同時に進める対戦数 [既定: config の train.num_envs]
    #[arg(long)]
    envs: Option<usize>,
    /// libtorch のスレッド数 [既定: 論理コア数の半分]
    #[arg(long)]
    threads: Option<usize>,
    /// 乱数シード [既定: ランダム]
    #[arg(long)]
    seed: Option<u64>,
    /// 設定ファイル。ゲームルールをここから読む (盤面の大きさだけは対戦する盤面に合わせる) [既定: learn/config.yaml]
    #[arg(long)]
    config: Option<PathBuf>,
    /// モデルの置き場所 [既定: model/]
    #[arg(long)]
    out_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = Cli::parse();
    match cli.command.take() {
        Some(Command::Versus(args)) => versus(args),
        None => train(cli),
    }
}

fn repo_dir() -> &'static Path {
    let crate_dir = Path::new(CRATE_DIR);
    crate_dir.parent().unwrap_or(crate_dir)
}

/// (設定ファイル, モデルの置き場所)。指定が無ければ既定の場所
fn resolve_paths(config: Option<PathBuf>, out_dir: Option<PathBuf>) -> (PathBuf, PathBuf) {
    (
        config.unwrap_or_else(|| Path::new(CRATE_DIR).join("config.yaml")),
        out_dir.unwrap_or_else(|| repo_dir().join("model")),
    )
}

/// libtorch のスレッド数を決めて設定する。小さなネットワークでは論理コアをすべて使うとかえって遅くなるので既定は半分
fn set_threads(threads: Option<usize>) -> usize {
    let threads = threads.unwrap_or_else(|| {
        let logical = std::thread::available_parallelism().map_or(1, |n| n.get());
        (logical / 2).max(1)
    });
    tch::set_num_threads(threads as i32);
    threads
}

fn train(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let repo_dir = repo_dir();
    let custom_out_dir = cli.out_dir.is_some();
    let (config_path, out_dir) = resolve_paths(cli.config, cli.out_dir);

    let mut config = Config::load(&config_path)?;
    let size = export::size_label(&config.game.grid);
    let model_file = export::model_file_name(&config.game.grid);
    let recent_dir = out_dir.join(RECENT_DIR);
    let recent_model = recent_dir.join(&model_file);
    if let Some(envs) = cli.envs {
        config.train.num_envs = envs.max(1);
    }
    let threads = set_threads(cli.threads);
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

    let resuming = resume.is_some();
    let mut trainer = Trainer::new(config.clone(), device, seed, resume)?;
    let previous_model = match save_branch_model_as_previous(&out_dir, &model_file) {
        Ok(path) => {
            notices.push(format!(
                "{PREVIOUS_MODEL_BRANCH} ブランチのモデルを旧モデルとして置きました: {} (`just versus` で新モデルと対戦できます)",
                path.display()
            ));
            Some(path)
        }
        Err(e) => {
            notices.push(format!("警告: 旧モデルを置けませんでした: {e}"));
            None
        }
    };
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let eval = match previous_model {
        _ if config.train.eval_games == 0 => {
            notices.push("旧モデルとの対戦      : しない (train.eval_games が 0)".to_string());
            None
        }
        None => {
            notices.push(
                "警告: 旧モデルが無いので、旧モデルとの対戦による勝率の記録はしません".to_string(),
            );
            None
        }
        Some(previous_model) => {
            // 出力先を指定した試しの学習では、docs/ の記録を汚さないよう出力先の下に書く
            let dir = if custom_out_dir {
                out_dir.join("training-progress")
            } else {
                repo_dir.join(PROGRESS_DIR)
            };
            let progress = ProgressFiles::new(&dir, &size);
            let resumed_games = resuming.then(|| trainer.info().games);
            if let Some(message) = progress.prepare(resumed_games, &timestamp)? {
                notices.push(message);
            }
            notices.push(format!(
                "旧モデルとの対戦      : {} 試合ごとの保存のたびに {} 試合。勝率を {} に足し、{} のグラフを描き直す",
                thousands(config.train.save_interval_games),
                thousands(config.train.eval_games),
                progress.csv.display(),
                progress.png.display()
            ));
            Some(EvalTarget {
                previous_model,
                progress,
            })
        }
    };
    let grid_len = GRID_CHANNELS * (config.game.grid.width * config.game.grid.height) as usize;
    let replay_mb = (config.replay_capacity()
        * ReplayBuffer::bytes_per_transition(grid_len, VECTOR_FEATURES)) as f64
        / 1e6;
    let resumed = trainer.info();
    let force = &config.reward.force;

    let mut log = TrainLog::create(&repo_dir.join(format!("log/train-{timestamp}")))?;
    let mut lines = vec![
        "DuelSnake-AI 自己対戦学習".to_string(),
        format!("  設定       : {}", config_path.display()),
        format!(
            "  盤面       : {size}{}",
            if config.board_size().is_some() {
                format!(" (config.yaml の board_sizes.{size} の設定を使う)")
            } else {
                " (board_sizes に設定が無いので model と train の値を使う)".to_string()
            }
        ),
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
            "  引力       : リンゴ {} / 金のリンゴ {} (range {})",
            force.normal_apple_attraction, force.gold_apple_attraction, force.attraction_range
        ),
        format!(
            "  斥力       : 毒リンゴ {} / お邪魔ブロック {} (range {})",
            force.poison_apple_repulsion, force.jam_block_repulsion, force.repulsion_range
        ),
    ];
    if cli.auto_commit {
        lines.push(format!(
            "  自動コミット: 通算 {} 試合ごとに最新モデルをコミットしてプッシュ",
            thousands(config.train.commit_interval_games)
        ));
    }
    if cli.commit_on_interrupt {
        lines
            .push("  中断時     : Ctrl+C で中断したら最新モデルをコミットしてプッシュ".to_string());
    }
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
    let on_stop = if cli.commit_on_interrupt {
        "保存し、コミットしてプッシュして"
    } else {
        "保存して"
    };
    println!("Ctrl+C で中断するとモデルを{on_stop}終了します");

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
        eval,
    };
    let commit = CommitPolicy {
        periodic: cli.auto_commit,
        on_interrupt: cli.commit_on_interrupt,
    };
    trainer.run(&running, cli.games, &paths, commit, &mut log)?;
    println!("学習を終了しました");
    log.event("学習を終了しました")?;
    Ok(())
}

/// main ブランチの最新モデル (model/recent-model/<model_file>) を <out_dir>/previous-model/ に書き出し、そのパスを返す。
/// 作業中のブランチのモデルではなく、いつも main のモデルを旧モデルにする。
/// 一時ファイルに書いてから置き換えるので、途中で止まっても壊れたファイルが残らない
fn save_branch_model_as_previous(out_dir: &Path, model_file: &str) -> Result<PathBuf, String> {
    let source = format!("{BRANCH_MODEL_DIR}/{model_file}");
    let content = git::show_file(repo_dir(), PREVIOUS_MODEL_BRANCH, &source).map_err(|e| {
        format!("{PREVIOUS_MODEL_BRANCH} ブランチの {source} を取り出せません ({e})")
    })?;
    let to = out_dir.join(PREVIOUS_DIR).join(model_file);
    let write = || -> std::io::Result<()> {
        fs::create_dir_all(out_dir.join(PREVIOUS_DIR))?;
        let tmp = to.with_extension("json.tmp");
        fs::write(&tmp, &content)?;
        fs::rename(&tmp, &to)
    };
    write().map_err(|e| format!("{} に書けません: {e}", to.display()))?;
    Ok(to)
}

/// 旧モデルと新モデルを対戦させ、結果を表示して log/versus-<日時>.txt にも残す。
/// 盤面を指定しなければ、旧モデル (main) と新モデルがそろうすべての盤面で対戦する
fn versus(args: VersusArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, out_dir) = resolve_paths(args.config, args.out_dir);
    let config = Config::load(&config_path)?;
    let mut lines = Vec::new();
    let sizes = match (args.size, args.new.as_ref().or(args.old.as_ref())) {
        (Some(size), _) => vec![size],
        (None, Some(path)) => {
            let grid = ModelFile::read(path)?.game.grid;
            vec![(grid.width, grid.height)]
        }
        (None, None) => {
            let (sizes, skipped) = paired_sizes(&out_dir);
            if !skipped.is_empty() {
                lines.push(format!(
                    "ペアがそろわないので対戦しない盤面: {}",
                    skipped.join(", ")
                ));
            }
            if sizes.is_empty() {
                return Err(format!(
                    "{PREVIOUS_MODEL_BRANCH} ブランチと {} の両方にモデルがある盤面がありません。--size か --old / --new で指定してください",
                    out_dir.join(RECENT_DIR).display()
                )
                .into());
            }
            sizes
        }
    };
    let envs = args.envs.unwrap_or(config.train.num_envs).max(1);
    let threads = set_threads(args.threads);
    let device = Device::cuda_if_available();
    let seed = args.seed.unwrap_or_else(rand::random);
    lines.insert(
        0,
        format!(
            "DuelSnake-AI 旧モデル vs 新モデル (盤面 {} / 各 {} 試合 / デバイス {device:?} / スレッド {threads} / 同時対戦 {envs} / シード {seed})",
            sizes.iter().map(|&s| size_text(s)).collect::<Vec<_>>().join(", "),
            thousands(args.games)
        ),
    );
    for line in &lines {
        println!("{line}");
    }

    let mut results = Vec::new();
    for &(width, height) in &sizes {
        let mut game = config.game.clone();
        (game.grid.width, game.grid.height) = (width, height);
        let rules = Rules::from_config(&game);
        let model_file = export::model_file_name(&game.grid);
        let mut section = vec![
            String::new(),
            format!("=== 盤面 {}", size_text((width, height))),
        ];
        let old = match &args.old {
            Some(path) => path.clone(),
            None => {
                // main のモデルが変わっていることがあるので、対戦の前に取り出し直す
                let path = out_dir.join(PREVIOUS_DIR).join(&model_file);
                match save_branch_model_as_previous(&out_dir, &model_file) {
                    Ok(_) => {}
                    Err(e) if path.exists() => section.push(format!(
                        "警告: 旧モデルを更新できませんでした。前に置いた旧モデルを使います: {e}"
                    )),
                    Err(e) => {
                        return Err(format!(
                            "旧モデルがありません: {e}。ほかのモデルと比べるなら --old で指定してください"
                        )
                        .into())
                    }
                }
                path
            }
        };
        let new = args
            .new
            .clone()
            .unwrap_or_else(|| out_dir.join(RECENT_DIR).join(&model_file));
        let contenders: [Contender; CONTENDERS] = [
            Contender::load(&old, &rules, device)?,
            Contender::load(&new, &rules, device)?,
        ];
        for (c, label) in [(OLD, "旧モデル"), (NEW, "新モデル")] {
            if contenders[c].game != game {
                section.push(format!(
                    "注意: {label}を学習したときのゲームルールが config.yaml と異なります。config.yaml のルールで対戦します"
                ));
            }
        }
        let tally = arena::play(&contenders, rules, args.games, envs, seed, device, true);
        section.extend(arena::report(&contenders, &tally));
        for line in &section {
            println!("{line}");
        }
        lines.extend(section);
        results.push((size_text((width, height)), tally));
    }

    let summary = arena::summary(&results);
    println!();
    for line in &summary {
        println!("{line}");
    }
    lines.push(String::new());
    lines.extend(summary);
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let log_path = repo_dir().join(format!("log/versus-{timestamp}.txt"));
    match fs::write(&log_path, lines.join("\n") + "\n") {
        Ok(()) => println!("結果を保存しました: {}", log_path.display()),
        Err(e) => println!(
            "警告: 結果を {} に保存できませんでした: {e}",
            log_path.display()
        ),
    }
    Ok(())
}

/// 盤面の大きさの指定 (例: 8-8、12-10)。8x8 の形 (モデルのファイル名と同じ) でもよい。数が 1 つだけなら正方形
fn parse_size(text: &str) -> Result<(i32, i32), String> {
    let number = |s: &str| s.trim().parse::<i32>().ok().filter(|&n| n > 0);
    let size = match text.split_once(['-', 'x', 'X']) {
        Some((w, h)) => number(w).zip(number(h)),
        None => number(text).map(|n| (n, n)),
    };
    size.ok_or_else(|| format!("盤面は 8-8 や 12-10 の形で指定してください: {text}"))
}

/// 画面とログに出す盤面の大きさ (例: 8-8)
fn size_text((width, height): (i32, i32)) -> String {
    format!("{width}-{height}")
}

/// 旧モデル (main ブランチ) と新モデル (<out_dir>/recent-model/) の両方がある盤面を、小さい順に返す。
/// 2 つ目は片方にしか無い盤面の説明
fn paired_sizes(out_dir: &Path) -> (Vec<(i32, i32)>, Vec<String>) {
    let parse_all = |labels: Vec<String>| -> Vec<(i32, i32)> {
        let mut sizes: Vec<_> = labels.iter().filter_map(|l| parse_size(l).ok()).collect();
        sizes.sort_by_key(|&(w, h)| (w * h, w));
        sizes
    };
    let mut notes = Vec::new();
    let main_files = git::list_files(repo_dir(), PREVIOUS_MODEL_BRANCH, BRANCH_MODEL_DIR)
        .unwrap_or_else(|e| {
            notes.push(format!(
                "{PREVIOUS_MODEL_BRANCH} ブランチのモデルの一覧を読めません ({e})"
            ));
            Vec::new()
        });
    let on_main = parse_all(
        main_files
            .iter()
            .filter_map(|name| export::size_from_file_name(name).map(str::to_string))
            .collect(),
    );
    let recent = parse_all(model_sizes_in(&out_dir.join(RECENT_DIR)));
    let paired = recent
        .iter()
        .copied()
        .filter(|s| on_main.contains(s))
        .collect();
    let mut skipped = notes;
    skipped.extend(
        recent
            .iter()
            .filter(|s| !on_main.contains(s))
            .map(|&s| format!("{} ({PREVIOUS_MODEL_BRANCH} に無い)", size_text(s))),
    );
    skipped.extend(
        on_main
            .iter()
            .filter(|s| !recent.contains(s))
            .map(|&s| format!("{} (新モデルが無い)", size_text(s))),
    );
    (paired, skipped)
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
    if !model.uses_full_observation() {
        return Err(format!(
            "再開元モデル {} の観測 (盤面 {} チャネル / 状態 {} 個) が今の観測 ({GRID_CHANNELS} チャネル / {VECTOR_FEATURES} 個) と異なるので、学習を再開できません。`just train` で新しく学習してください (このモデルとは `just versus` で対戦できます)",
            path.display(),
            model.observation.grid_channels.len(),
            model.observation.vector_features.len()
        )
        .into());
    }
    notices.push(format!("学習を再開します: {}", path.display()));
    if model.game != config.game {
        notices.push("注意: 再開元モデルと config.yaml のゲームルールが異なります。現在の config.yaml のルールで学習を続けます".to_string());
    }
    Ok(model)
}

/// `dir` にある盤面サイズごとのモデルの盤面サイズ (例: 16x16)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_board_sizes() {
        assert_eq!(parse_size("8-8"), Ok((8, 8)));
        assert_eq!(parse_size("12-10"), Ok((12, 10)));
        assert_eq!(parse_size("8x8"), Ok((8, 8)));
        assert_eq!(parse_size("12X10"), Ok((12, 10)));
        assert_eq!(parse_size("16"), Ok((16, 16)));
        assert_eq!(size_text((12, 10)), "12-10");
        for bad in ["", "-8", "8-", "0-8", "8-8-8", "x8", "8x", "8x8x8", "abc"] {
            assert!(parse_size(bad).is_err(), "{bad}");
        }
    }
}
