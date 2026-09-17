mod agent;
mod config;
mod env;
mod memory;

use agent::DQNAgent;
use config::GameConfig;
use env::game::GameEnv;
use env::snake::Direction;
use env::tensor::field_to_tensor;

use chrono::Local;
use std::env as std_env;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std_env::args().collect();
    let mode = args
        .iter()
        .position(|r| r == "--mode")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("half-memory");

    println!("DuelSnake-AI RL Training Started");
    let config = GameConfig::load("../model/config.yaml")?;

    let env_mem_bytes = std::mem::size_of::<GameEnv>();
    let workers = memory::calculate_workers(mode, env_mem_bytes);
    println!("Mode: {}, Active Workers: {}", mode, workers);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n[SIGINT] 中断シグナルを受信。エピソード完了後にモデルを保存します...");
        r.store(false, Ordering::SeqCst);
    })?;

    let agent = DQNAgent::new(&config.agent);
    let mut episode = 0;

    while running.load(Ordering::SeqCst) {
        episode += 1;
        let mut game = GameEnv::new(config.clone());

        while game.time_remaining_steps > 0 && game.player_field.snake.is_alive {
            let p_tensor = field_to_tensor(
                &game.player_field,
                config.grid.width as i64,
                config.grid.height as i64,
            );
            let ai_tensor = field_to_tensor(
                &game.ai_field,
                config.grid.width as i64,
                config.grid.height as i64,
            );

            let p_action = agent.select_action(
                &p_tensor,
                config.agent.epsilon,
                config.agent.random_action_bound,
            );
            let ai_action = agent.select_action(
                &ai_tensor,
                config.agent.epsilon,
                config.agent.random_action_bound,
            );

            let (p_dir, p_use) = parse_action(p_action, game.player_field.snake.dir);
            let (ai_dir, ai_use) = parse_action(ai_action, game.ai_field.snake.dir);

            game.step(p_dir, p_use, ai_dir, ai_use);
        }

        if episode % 100 == 0 {
            println!(
                "Completed Episode: {}, Player Score: {}",
                episode, game.player_field.snake.score
            );
        }
    }

    // --- 1. .pt モデルの保存（バックアップ ＋ recent） ---
    println!("Saving trained model...");
    fs::create_dir_all("../model/recent-model")?;
    fs::create_dir_all("../model/models")?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_path = format!("../model/models/snake-model-{}.pt", timestamp);
    let recent_pt_path = "../model/recent-model/snake-model.pt";
    let recent_onnx_path = "../model/recent-model/snake-model.onnx";

    agent.vs.save(&backup_path)?;
    agent.vs.save(recent_pt_path)?;

    println!("Saved backup model: {}", backup_path);
    println!("Updated recent model: {}", recent_pt_path);

    // --- 2. 成果物 (.onnx) への変換処理 ---
    println!("Converting recent model to ONNX format...");
    let script_path = "../model/convert-to-onnx.py";

    let status = Command::new("python3")
        .arg(script_path)
        .arg(recent_pt_path)
        .arg(recent_onnx_path)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Successfully generated ONNX model: {}", recent_onnx_path);
        }
        _ => {
            eprintln!("Warning: Failed to convert model to ONNX. Ensure python3, torch, and script exist.");
        }
    }

    println!("Training session finished safely.");
    Ok(())
}

fn parse_action(action: i64, current_dir: Direction) -> (Direction, bool) {
    match action {
        0 => (Direction::Up, false),
        1 => (Direction::Down, false),
        2 => (Direction::Left, false),
        3 => (Direction::Right, false),
        4 => (current_dir, true),
        _ => (current_dir, false),
    }
}