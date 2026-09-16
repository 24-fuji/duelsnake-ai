use std::fs;
use std::path::Path;
use chrono::Local;

pub fn save_and_update_model(source_temp_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let models_dir = Path::new("../model/models");
    let recent_dir = Path::new("../model/recent-model");

    // ディレクトリが存在しない場合は作成
    fs::create_dir_all(models_dir)?;
    fs::create_dir_all(recent_dir)?;

    // 1. タイムスタンプ付きでバックアップ保存
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_filename = format!("snake-model-{}.onnx", timestamp);
    let backup_path = models_dir.join(backup_filename);

    fs::copy(source_temp_path, &backup_path)?;
    println!("Saved backup model: {:?}", backup_path);

    // 2. recent-model への上書きコピー（常に最新1つ）
    let recent_path = recent_dir.join("snake-model.onnx");
    fs::copy(source_temp_path, &recent_path)?;
    println!("Updated recent model: {:?}", recent_path);

    Ok(())
}

pub fn get_recent_model_path() -> Option<String> {
    let recent_path = Path::new("../model/recent-model/snake-model.onnx");
    
    // ファイルが存在し、かつ0バイトでないか確認
    if recent_path.exists() {
        if let Ok(metadata) = fs::metadata(recent_path) {
            if metadata.len() > 0 {
                return Some(recent_path.to_str().unwrap().to_string());
            }
        }
    }
    None
}