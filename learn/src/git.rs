//! 学習中のモデルを git にコミットしてプッシュする

use std::path::Path;
use std::process::{Command, Stdio};

/// `file` だけを `message` でコミットし、今のブランチをプッシュする。
/// ほかにステージしてある変更はコミットに含めない
pub fn commit_and_push(file: &Path, message: &str) -> Result<(), String> {
    let (Some(dir), Some(name)) = (file.parent(), file.file_name().and_then(|n| n.to_str())) else {
        return Err(format!("{} はファイルのパスではありません", file.display()));
    };
    git(dir, &["add", "--", name])?;
    git(dir, &["commit", "--only", "-m", message, "--", name])?;
    git(dir, &["push"])
}

fn git(dir: &Path, args: &[&str]) -> Result<(), String> {
    // 学習の画面を乱さないよう、入力は求めず、出力は失敗したときだけ返す
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("git を実行できません: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let detail = if stderr.trim().is_empty() {
        String::from_utf8_lossy(&output.stdout)
    } else {
        stderr
    };
    Err(format!(
        "git {} が失敗しました: {}",
        args[0],
        detail.trim().replace('\n', " / ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn run(dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {args:?}: {output:?}");
        String::from_utf8(output.stdout).unwrap()
    }

    /// プッシュ先の空リポジトリと、そこへプッシュできる作業リポジトリを作る
    fn repos() -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "duelsnake-git-test-{}-{}",
            std::process::id(),
            chrono::Local::now().format("%H%M%S%f")
        ));
        let (remote, work) = (root.join("remote.git"), root.join("work"));
        fs::create_dir_all(&remote).unwrap();
        fs::create_dir_all(work.join("model")).unwrap();
        run(&remote, &["init", "-q", "--bare"]);
        run(&work, &["init", "-q"]);
        for (key, value) in [
            ("user.name", "test"),
            ("user.email", "test@example.com"),
            ("commit.gpgsign", "false"),
        ] {
            run(&work, &["config", key, value]);
        }
        fs::write(work.join("README"), "first").unwrap();
        run(&work, &["add", "README"]);
        run(&work, &["commit", "-q", "-m", "first"]);
        run(
            &work,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        run(&work, &["push", "-q", "-u", "origin", "HEAD"]);
        (remote, work)
    }

    #[test]
    fn commits_only_the_model_and_pushes() {
        let (remote, work) = repos();
        // 別にステージしてある変更はコミットに入らず、ステージされたまま残る
        fs::write(work.join("README"), "edited").unwrap();
        run(&work, &["add", "README"]);
        let model = work.join("model/snake-model-16x16.json");
        fs::write(&model, "{}").unwrap();

        commit_and_push(&model, "model update").unwrap();

        let pushed = run(&remote, &["log", "-1", "--name-only", "--format=%s"]);
        assert_eq!(
            pushed.trim(),
            "model update\n\nmodel/snake-model-16x16.json"
        );
        assert_eq!(
            run(&work, &["diff", "--cached", "--name-only"]).trim(),
            "README"
        );
        fs::remove_dir_all(remote.parent().unwrap()).unwrap();
    }

    #[test]
    fn reports_failure_outside_a_repository() {
        let dir = std::env::temp_dir();
        let err = commit_and_push(&dir.join("no-such-model.json"), "model update").unwrap_err();
        assert!(err.starts_with("git add が失敗しました"), "{err}");
    }
}
