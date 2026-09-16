mod env;
mod export;

use env::game::GameEnv;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting DuelSnake-AI RL Training...");

    // --- 1. Ctrl+C フラグの設定 ---
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\n[Interrupt] Ctrl+C received! Finishing the current episode before exit...");
        r.store(false, Ordering::SeqCst);
    })?;

    // --- 2. 既存モデルの読み込みチェック ---
    if let Some(recent_model) = export::get_recent_model_path() {
        println!("Found valid existing model: {}. Resuming training...", recent_model);
        // TODO: ここでモデルのウェイトをロードする処理を呼び出す
    } else {
        println!("No valid existing model found. Training from scratch...");
    }

    let mut env = GameEnv::new(15);
    let max_episodes = 10000;

    // --- 3. 学習メインループ ---
    for episode in 1..=max_episodes {
        // Ctrl+C が押されていたら現エピソード完了時点でループを脱出
        if !running.load(Ordering::SeqCst) {
            println!("Stopping training gracefully at episode {}...", episode - 1);
            break;
        }

        // 1エピソードの実行（ゲームオーバーまで）
        let mut episode_over = false;
        while !episode_over {
            env.step(0.1);
            
            // ゲームオーバーまたは30秒経過の判定
            if env.time_remaining <= 0.0 || !env.player_snake.is_alive || !env.ai_snake.is_alive {
                episode_over = true;
            }
        }

        // 環境のリセット
        env = GameEnv::new(15);
    }

    // --- 4. 学習結果の保存処理 ---
    println!("Exporting trained model...");
    // 一時出力ファイルパスを渡してバックアップ＆recent更新を実行
    let temp_output = "/tmp/temp-snake-model.onnx";
    
    // TODO: 実際のニューラルネットワークのパラメータを temp_output にエクスポートする処理
    std::fs::write(temp_output, b"dummy onnx content")?; // デモ用のダミー書き込み

    export::save_and_update_model(temp_output)?;

    println!("Training session finished safely.");
    Ok(())
}