use super::game::{Field, ItemType};
use tch::Tensor;

pub const NUM_CHANNELS: i64 = 8;

pub fn field_to_tensor(field: &Field, width: i64, height: i64) -> Tensor {
    let tensor = Tensor::zeros(&[NUM_CHANNELS, height, width], tch::kind::FLOAT_CPU);

    // チャネル 0: 自ヘビの頭
    if let Some(head) = field.snake.body.front() {
        let _ = tensor.get(0).get(head.y as i64).get(head.x as i64).fill_(1.0);
    }

    // チャネル 1: 自ヘビの胴体
    for (idx, pos) in field.snake.body.iter().enumerate() {
        if idx > 0 {
            let _ = tensor.get(1).get(pos.y as i64).get(pos.x as i64).fill_(1.0);
        }
    }

    // チャネル 2〜6: 各種アイテム
    for item in &field.items {
        let channel_idx = match item.item_type {
            ItemType::NormalApple => 2,
            ItemType::GoldApple => 3,
            ItemType::PoisonApple => 4,
            ItemType::BlockClear => 5,
            ItemType::BlockJam => 6,
        };
        let _ = tensor.get(channel_idx).get(item.pos.y as i64).get(item.pos.x as i64).fill_(1.0);
    }

    // チャネル 7: お邪魔ブロック
    for block in &field.obstacle_blocks {
        let _ = tensor.get(7).get(block.y as i64).get(block.x as i64).fill_(1.0);
    }

    tensor
}