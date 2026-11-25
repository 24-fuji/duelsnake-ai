//! 学習中のモデルを git にコミットしてプッシュする。旧モデルにする main のモデルも git から取り出す

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

/// ブランチなど `rev` の時点の `path` (リポジトリ直下からのパス) の中身を返す。`dir` はリポジトリの中のどこでもよい
pub fn show_file(dir: &Path, rev: &str, path: &str) -> Result<Vec<u8>, String> {
    git_output(dir, &["show", &format!("{rev}:{path}")])
}

/// ブランチなど `rev` の時点で、フォルダ `path` (リポジトリ直下から) にあるファイルの名前
pub fn list_files(dir: &Path, rev: &str, path: &str) -> Result<Vec<String>, String> {
    let tree = format!("{rev}:{path}");
    let output = git_output(dir, &["ls-tree", "--name-only", &tree])?;
    Ok(String::from_utf8_lossy(&output)
        .lines()
        .map(str::to_string)
        .collect())
}

fn git(dir: &Path, args: &[&str]) -> Result<(), String> {
    git_output(dir, args).map(|_| ())
}

/// git を実行し、成功すれば標準出力を返す
fn git_output(dir: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    // 学習の画面を乱さないよう、入力は求めず、出力は呼び出し元に返すか、失敗したときのエラーに入れる
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("git を実行できません: {e}"))?;
    if output.status.success() {
        return Ok(output.stdout);
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
    fn shows_a_file_on_another_branch() {
        let (remote, work) = repos();
        run(&work, &["branch", "base"]);
        fs::write(work.join("README"), "edited").unwrap();
        run(&work, &["commit", "-q", "-am", "edit"]);

        // 作業中のファイルではなく、指定したブランチの時点の中身を返す
        assert_eq!(show_file(&work, "base", "README").unwrap(), b"first");
        assert_eq!(
            show_file(&work.join("model"), "base", "README").unwrap(),
            b"first"
        );
        assert_eq!(list_files(&work, "base", "").unwrap(), vec!["README"]);
        assert!(list_files(&work, "base", "no-such-dir").is_err());
        let err = show_file(&work, "base", "no-such-file").unwrap_err();
        assert!(err.starts_with("git show が失敗しました"), "{err}");
        fs::remove_dir_all(remote.parent().unwrap()).unwrap();
    }

    #[test]
    fn reports_failure_outside_a_repository() {
        let dir = std::env::temp_dir();
        let err = commit_and_push(&dir.join("no-such-model.json"), "model update").unwrap_err();
        assert!(err.starts_with("git add が失敗しました"), "{err}");
    }
}
