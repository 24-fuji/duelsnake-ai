//! 学習の経過の記録。モデルを保存するたびに旧モデル (main のモデル) と対戦させた結果を CSV に足し、勝率のグラフ (PNG) を描き直す。
//!
//! CSV は git 管理外の元データで、PNG は git で管理して外から見られるようにする。
//! 新しく学習を始めたときは前の CSV を archive/ に移し、再開したときは再開した時点より後の行を捨てて続きから足す

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// 列の並び。グラフを描くスクリプト (learn/scripts/plot_winrate.py) もこの名前で読む
const HEADER: &str = "time,games,decisions,updates,epsilon,eval_games,new_wins,old_wins,draws,win_rate,win_rate_ci95,new_avg_score,old_avg_score,new_death_rate,old_death_rate,avg_ticks,old_model_games,old_model_created_at";

/// グラフを描くスクリプト
const PLOT_SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/plot_winrate.py");

/// 1 回の評価 (旧モデルとの対戦) の結果
#[derive(Debug, Clone, PartialEq)]
pub struct EvalRow {
    pub time: String,
    pub games: u64,
    pub decisions: u64,
    pub updates: u64,
    pub epsilon: f64,
    pub eval_games: u64,
    pub new_wins: u64,
    pub old_wins: u64,
    pub draws: u64,
    pub win_rate: f64,
    pub win_rate_ci95: f64,
    pub new_avg_score: f64,
    pub old_avg_score: f64,
    pub new_death_rate: f64,
    pub old_death_rate: f64,
    pub avg_ticks: f64,
    pub old_model_games: u64,
    pub old_model_created_at: String,
}

impl EvalRow {
    fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{:.4},{},{},{},{},{:.4},{:.4},{:.3},{:.3},{:.4},{:.4},{:.1},{},{}",
            self.time,
            self.games,
            self.decisions,
            self.updates,
            self.epsilon,
            self.eval_games,
            self.new_wins,
            self.old_wins,
            self.draws,
            self.win_rate,
            self.win_rate_ci95,
            self.new_avg_score,
            self.old_avg_score,
            self.new_death_rate,
            self.old_death_rate,
            self.avg_ticks,
            self.old_model_games,
            // 日時に , は入らないが、念のため CSV が崩れないようにする
            self.old_model_created_at.replace(',', " ")
        )
    }
}

/// 1 つの盤面サイズの記録ファイル
#[derive(Debug, Clone)]
pub struct ProgressFiles {
    pub csv: PathBuf,
    pub png: PathBuf,
}

impl ProgressFiles {
    /// `dir` の下の winrate-<size>.csv と .png
    pub fn new(dir: &Path, size: &str) -> Self {
        Self {
            csv: dir.join(format!("winrate-{size}.csv")),
            png: dir.join(format!("winrate-{size}.png")),
        }
    }

    /// 学習を始める前に CSV を整える。返す文は、何かしたときのお知らせ。
    /// `resumed_games` が None (新しく学習する) なら前の CSV を archive/ に移し、
    /// Some(n) (n 試合目から再開する) なら n 試合より後の行を捨てる
    pub fn prepare(
        &self,
        resumed_games: Option<u64>,
        timestamp: &str,
    ) -> io::Result<Option<String>> {
        if !self.csv.exists() {
            return Ok(None);
        }
        let Some(resumed) = resumed_games else {
            let dir = self.csv.with_file_name("archive");
            fs::create_dir_all(&dir)?;
            let stem = self
                .csv
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("winrate");
            let to = dir.join(format!("{stem}-{timestamp}.csv"));
            fs::rename(&self.csv, &to)?;
            return Ok(Some(format!(
                "新しく学習を始めるので、前の勝率の記録を {} に移しました",
                to.display()
            )));
        };
        let content = fs::read_to_string(&self.csv)?;
        let mut lines = content.lines();
        let header = lines.next().unwrap_or(HEADER);
        let rows: Vec<&str> = lines.filter(|l| !l.trim().is_empty()).collect();
        let kept: Vec<&str> = rows
            .iter()
            .copied()
            .filter(|row| row_games(row).is_some_and(|g| g <= resumed))
            .collect();
        if kept.len() == rows.len() {
            return Ok(None);
        }
        write_atomic(
            &self.csv,
            &std::iter::once(header)
                .chain(kept.iter().copied())
                .map(|l| format!("{l}\n"))
                .collect::<String>(),
        )?;
        Ok(Some(format!(
            "再開元より後の勝率の記録 {} 行を {} から消しました",
            rows.len() - kept.len(),
            self.csv.display()
        )))
    }

    /// CSV に 1 行足す。無ければ見出しから作る
    pub fn append(&self, row: &EvalRow) -> io::Result<()> {
        if let Some(dir) = self.csv.parent() {
            fs::create_dir_all(dir)?;
        }
        let new_file = !self.csv.exists();
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.csv)?;
        if new_file {
            writeln!(file, "{HEADER}")?;
        }
        writeln!(file, "{}", row.to_csv())
    }

    /// CSV から PNG のグラフを描き直す (python3 と matplotlib を使う)
    pub fn plot(&self) -> Result<(), String> {
        plot(&self.csv, &self.png)
    }
}

/// CSV の行の games 列 (2 列目)
fn row_games(row: &str) -> Option<u64> {
    row.split(',').nth(1)?.trim().parse().ok()
}

fn write_atomic(path: &Path, content: &str) -> io::Result<()> {
    let tmp = path.with_extension("csv.tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)
}

/// `csv` の勝率の推移を `png` に描く
pub fn plot(csv: &Path, png: &Path) -> Result<(), String> {
    let output = Command::new("python3")
        .arg(PLOT_SCRIPT)
        .arg(csv)
        .arg(png)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("python3 を実行できません: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "グラフを描けませんでした ({}): {}",
        PLOT_SCRIPT,
        String::from_utf8_lossy(&output.stderr)
            .trim()
            .lines()
            .last()
            .unwrap_or("")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "duelsnake-progress-{name}-{}-{}",
            std::process::id(),
            chrono::Local::now().format("%H%M%S%f")
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn row(games: u64) -> EvalRow {
        EvalRow {
            time: "2026-11-26 10:00:00".to_string(),
            games,
            decisions: games * 100,
            updates: games * 3,
            epsilon: 0.01,
            eval_games: 1000,
            new_wins: 600,
            old_wins: 390,
            draws: 10,
            win_rate: 0.605,
            win_rate_ci95: 0.03,
            new_avg_score: 25.0,
            old_avg_score: 24.0,
            new_death_rate: 0.3,
            old_death_rate: 0.6,
            avg_ticks: 190.0,
            old_model_games: 2_396_304,
            old_model_created_at: "2026-10-09T06:03:50+09:00".to_string(),
        }
    }

    fn games_in(csv: &Path) -> Vec<u64> {
        fs::read_to_string(csv)
            .unwrap()
            .lines()
            .skip(1)
            .filter_map(row_games)
            .collect()
    }

    #[test]
    fn appends_rows_under_one_header() {
        let dir = dir("append");
        let files = ProgressFiles::new(&dir, "8x8");
        files.append(&row(10_000)).unwrap();
        files.append(&row(20_000)).unwrap();
        let content = fs::read_to_string(&files.csv).unwrap();
        assert_eq!(content.lines().next(), Some(HEADER));
        assert_eq!(games_in(&files.csv), vec![10_000, 20_000]);
        // 列の数が見出しとそろっている
        let columns = HEADER.split(',').count();
        assert!(content.lines().all(|l| l.split(',').count() == columns));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn resuming_drops_rows_after_the_resume_point() {
        let dir = dir("resume");
        let files = ProgressFiles::new(&dir, "8x8");
        for games in [10_000, 20_000, 30_000] {
            files.append(&row(games)).unwrap();
        }
        assert_eq!(files.prepare(Some(30_000), "t").unwrap(), None);
        assert!(files.prepare(Some(20_500), "t").unwrap().is_some());
        assert_eq!(games_in(&files.csv), vec![10_000, 20_000]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn fresh_training_moves_the_old_record_to_archive() {
        let dir = dir("fresh");
        let files = ProgressFiles::new(&dir, "8x8");
        assert_eq!(files.prepare(None, "t").unwrap(), None);
        files.append(&row(10_000)).unwrap();
        assert!(files.prepare(None, "20261126-100000").unwrap().is_some());
        assert!(!files.csv.exists());
        let archived = dir.join("archive/winrate-8x8-20261126-100000.csv");
        assert_eq!(games_in(&archived), vec![10_000]);
        fs::remove_dir_all(dir).unwrap();
    }
}
