//! 学習の様子の出力。詳しい統計はログファイルに書き、コンソールには最下行の状況表示とお知らせだけを出す

use crate::env::game::Action;
use std::fs::File;
use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const ACTION_LABELS: [&str; Action::COUNT] = ["上", "下", "左", "右", "ブースト", "アイテム"];

/// ログ出力の間隔ごとの統計
pub struct StatsRow {
    pub games: u64,
    /// この間に終わった試合数
    pub window_games: u64,
    pub decisions: u64,
    pub updates: u64,
    pub epsilon: f64,
    pub loss: f64,
    /// 学習バッチでの max_a Q(s, a) の平均
    pub q_mean: f64,
    pub avg_score: f64,
    pub avg_length: f64,
    /// 1匹1試合あたりの引力・斥力による報酬
    pub avg_force_reward: f64,
    pub death_rate: f64,
    pub draw_rate: f64,
    pub avg_ticks: f64,
    pub games_per_sec: f64,
    /// ランダムでない行動の内訳 (Action::ALL の順)
    pub action_rates: [f64; Action::COUNT],
}

/// 人が読む用の `<base>.log` と、グラフ用の `<base>.csv`。書くたびに flush するので `tail -f` で追える
pub struct TrainLog {
    text_path: PathBuf,
    text: BufWriter<File>,
    csv: BufWriter<File>,
}

impl TrainLog {
    pub fn create(base: &Path) -> io::Result<Self> {
        if let Some(dir) = base.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text_path = base.with_extension("log");
        let text = BufWriter::new(File::create(&text_path)?);
        let mut csv = BufWriter::new(File::create(base.with_extension("csv"))?);
        write!(
            csv,
            "time,games,window_games,decisions,updates,epsilon,loss,q_mean,avg_score,avg_length,avg_force_reward,death_rate,draw_rate,avg_ticks,games_per_sec"
        )?;
        for action in Action::ALL {
            write!(csv, ",action_{}", action.name())?;
        }
        writeln!(csv)?;
        csv.flush()?;
        Ok(Self {
            text_path,
            text,
            csv,
        })
    }

    pub fn text_path(&self) -> &Path {
        &self.text_path
    }

    /// 時刻付きで1行書く
    pub fn event(&mut self, message: &str) -> io::Result<()> {
        writeln!(self.text, "{} {message}", now())?;
        self.text.flush()
    }

    pub fn stats(&mut self, s: &StatsRow) -> io::Result<()> {
        let actions: Vec<String> = ACTION_LABELS
            .iter()
            .zip(s.action_rates)
            .map(|(label, rate)| format!("{label} {:.0}%", rate * 100.0))
            .collect();
        self.event(&format!(
            "[{} 試合] ε {:.3} | loss {:.4} | 平均Q {:.3} | スコア {:.2} | 長さ {:.1} | 引力・斥力 {:+.3} | 死亡決着 {:.1}% | 引き分け {:.1}% | 平均 {:.0} ティック | {:.1} 試合/秒 (この間 {} 試合) | 行動 {}",
            thousands(s.games),
            s.epsilon,
            s.loss,
            s.q_mean,
            s.avg_score,
            s.avg_length,
            s.avg_force_reward,
            s.death_rate * 100.0,
            s.draw_rate * 100.0,
            s.avg_ticks,
            s.games_per_sec,
            thousands(s.window_games),
            actions.join(" "),
        ))?;

        write!(
            self.csv,
            "{},{},{},{},{},{:.4},{:.6},{:.4},{:.3},{:.2},{:.4},{:.4},{:.4},{:.1},{:.2}",
            now(),
            s.games,
            s.window_games,
            s.decisions,
            s.updates,
            s.epsilon,
            s.loss,
            s.q_mean,
            s.avg_score,
            s.avg_length,
            s.avg_force_reward,
            s.death_rate,
            s.draw_rate,
            s.avg_ticks,
            s.games_per_sec,
        )?;
        for rate in s.action_rates {
            write!(self.csv, ",{rate:.4}")?;
        }
        writeln!(self.csv)?;
        self.csv.flush()
    }
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// コンソール最下行の状況表示。端末でないとき (ファイルへのリダイレクトなど) は一定間隔で1行ずつ出す
pub struct StatusLine {
    interactive: bool,
    interval: Duration,
    last_shown: Option<Instant>,
    text: String,
}

impl StatusLine {
    pub fn new() -> Self {
        let interactive = io::stdout().is_terminal();
        Self {
            interactive,
            interval: Duration::from_millis(if interactive { 500 } else { 60_000 }),
            last_shown: None,
            text: String::new(),
        }
    }

    /// 前回の表示から表示間隔が過ぎたか
    pub fn due(&self) -> bool {
        self.last_shown
            .is_none_or(|shown| shown.elapsed() >= self.interval)
    }

    pub fn show(&mut self, text: String) {
        self.text = text;
        self.last_shown = Some(Instant::now());
        let mut out = io::stdout().lock();
        let _ = if self.interactive {
            write!(out, "\r\x1b[2K{}", self.text)
        } else {
            writeln!(out, "{}", self.text)
        };
        let _ = out.flush();
    }

    /// 状況表示の上にお知らせを1行出す
    pub fn message(&self, message: &str) {
        let mut out = io::stdout().lock();
        let _ = if self.interactive {
            write!(out, "\r\x1b[2K{message}\n{}", self.text)
        } else {
            writeln!(out, "{message}")
        };
        let _ = out.flush();
    }

    /// 状況表示を消す
    pub fn clear(&mut self) {
        if self.interactive && !self.text.is_empty() {
            let mut out = io::stdout().lock();
            let _ = write!(out, "\r\x1b[2K");
            let _ = out.flush();
        }
        self.text.clear();
    }
}

/// 3桁ごとにカンマで区切る
pub fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// 見やすさのため下の桁を切り捨てた概数。1万未満は百の位、それ以上は千の位で切り捨てる
pub fn approx_count(n: u64) -> String {
    let step = if n < 10_000 { 100 } else { 1_000 };
    thousands(n / step * step)
}

/// 時:分:秒
pub fn hms(d: Duration) -> String {
    let s = d.as_secs();
    format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_counts_for_display() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(1_234_567), "1,234,567");

        assert_eq!(approx_count(99), "0");
        assert_eq!(approx_count(1_234), "1,200");
        assert_eq!(approx_count(9_999), "9,900");
        assert_eq!(approx_count(12_345), "12,000");
        assert_eq!(approx_count(1_234_567), "1,234,000");
    }

    #[test]
    fn formats_elapsed_time() {
        assert_eq!(hms(Duration::from_secs(5)), "0:00:05");
        assert_eq!(hms(Duration::from_secs(3_723)), "1:02:03");
        assert_eq!(hms(Duration::from_secs(90_000)), "25:00:00");
    }
}
