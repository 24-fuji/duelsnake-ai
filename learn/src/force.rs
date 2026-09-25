//! 引力・斥力による報酬の補助。学習時だけ使い、ゲームのルールや Web 版には影響しない
//!
//! アイテムからヘビの頭に力が働いているとみなし、頭が動いたときに力がした仕事を報酬に足す。
//! 位置エネルギーは 引力なら -強さ * 近さ、斥力なら +強さ * 近さ で、近さは exp(-距離 / range)。
//! 仕事はアイテムの位置を止めたまま「動く前 − 動いた後」の位置エネルギーで求める。
//! 取ったアイテムは距離 0 まで近づいたものとして数え、新しく出たアイテムは次の移動から数える。
//! アイテムは取られるまで動かないので、近づいたり離れたりを繰り返しても報酬は増えない。

use crate::config::ForceConfig;
use crate::env::field::{Item, ItemType};
use crate::env::snake::Position;

#[derive(Debug, Clone, Copy)]
pub struct ForceField {
    range: f32,
    /// アイテムごとの強さ。引力は正、斥力は負
    normal_apple: f32,
    gold_apple: f32,
    poison_apple: f32,
}

impl ForceField {
    pub fn new(cfg: &ForceConfig) -> Self {
        Self {
            range: cfg.range,
            normal_apple: cfg.normal_apple_attraction,
            gold_apple: cfg.gold_apple_attraction,
            poison_apple: -cfg.poison_apple_repulsion,
        }
    }

    /// 力が1つでも働くか
    pub fn is_active(&self) -> bool {
        [self.normal_apple, self.gold_apple, self.poison_apple]
            .iter()
            .any(|&s| s != 0.0)
    }

    fn strength(&self, kind: ItemType) -> f32 {
        match kind {
            ItemType::NormalApple => self.normal_apple,
            ItemType::GoldApple => self.gold_apple,
            ItemType::PoisonApple => self.poison_apple,
            ItemType::BlockClear | ItemType::BlockJam => 0.0,
        }
    }

    /// 距離 0 で 1、range 離れるごとに 1/e 倍になる
    fn closeness(&self, item: Position, head: Position) -> f32 {
        (-(item.manhattan(head) as f32) / self.range).exp()
    }

    /// `items` を止めたまま頭が `from` から `to` へ動いたときに、力が頭にした仕事
    pub fn work(&self, items: &[Item], from: Position, to: Position) -> f32 {
        if from == to {
            return 0.0;
        }
        items
            .iter()
            .map(|item| {
                let strength = self.strength(item.kind);
                if strength == 0.0 {
                    return 0.0;
                }
                strength * (self.closeness(item.pos, to) - self.closeness(item.pos, from))
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> ForceField {
        ForceField::new(&ForceConfig {
            range: 4.0,
            normal_apple_attraction: 0.1,
            gold_apple_attraction: 0.3,
            poison_apple_repulsion: 0.2,
        })
    }

    fn pos(x: i32, y: i32) -> Position {
        Position { x, y }
    }

    fn item(kind: ItemType, x: i32, y: i32) -> Item {
        Item {
            pos: pos(x, y),
            kind,
        }
    }

    #[test]
    fn attraction_rewards_approaching_apples() {
        let f = field();
        let apple = [item(ItemType::NormalApple, 5, 0)];
        assert!(f.work(&apple, pos(0, 0), pos(1, 0)) > 0.0);
        assert!(f.work(&apple, pos(1, 0), pos(0, 0)) < 0.0);
        // 取る (距離 1 → 0) と、近さの差 1 - e^(-1/4) に強さを掛けた分
        let eat = f.work(&apple, pos(4, 0), pos(5, 0));
        assert!((eat - 0.1 * (1.0 - (-0.25f32).exp())).abs() < 1e-6);

        let gold = [item(ItemType::GoldApple, 5, 0)];
        assert!((f.work(&gold, pos(4, 0), pos(5, 0)) - 3.0 * eat).abs() < 1e-6);
    }

    #[test]
    fn repulsion_penalizes_approaching_poison() {
        let f = field();
        let poison = [item(ItemType::PoisonApple, 5, 0)];
        assert!(f.work(&poison, pos(3, 0), pos(4, 0)) < 0.0);
        assert!(f.work(&poison, pos(4, 0), pos(3, 0)) > 0.0);
    }

    #[test]
    fn round_trip_earns_nothing() {
        let f = field();
        let items = [
            item(ItemType::NormalApple, 2, 3),
            item(ItemType::GoldApple, 7, 1),
            item(ItemType::PoisonApple, 4, 4),
        ];
        let path = [pos(3, 3), pos(3, 2), pos(4, 2), pos(4, 3), pos(3, 3)];
        let total: f32 = path.windows(2).map(|w| f.work(&items, w[0], w[1])).sum();
        assert!(total.abs() < 1e-6, "{total}");
    }

    #[test]
    fn items_without_force_are_ignored() {
        let f = field();
        let items = [
            item(ItemType::BlockClear, 1, 0),
            item(ItemType::BlockJam, 1, 0),
        ];
        assert_eq!(f.work(&items, pos(0, 0), pos(1, 0)), 0.0);

        let off = ForceField::new(&ForceConfig {
            range: 4.0,
            normal_apple_attraction: 0.0,
            gold_apple_attraction: 0.0,
            poison_apple_repulsion: 0.0,
        });
        assert!(!off.is_active());
        assert!(field().is_active());
    }
}
