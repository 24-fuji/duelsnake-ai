mod env;
mod export;

use env::game::GameEnv;

fn main() {
    println!("Starting DuelSnake-AI RL Training...");
    let mut env = GameEnv::new(15);
    
    // 学習メインループ
    for epoch in 0..1000 {
        env.step(0.1);
        if env.time_remaining <= 0.0 {
            break;
        }
    }
    
    export::export_to_onnx("../model/snake-model.onnx");
}