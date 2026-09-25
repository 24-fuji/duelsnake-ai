//! 引力・斥力による報酬の補助。学習時だけ使い、ゲームのルールや Web 版には影響しない
//!
//! アイテムやお邪魔ブロックからヘビの頭に力が働いているとみなし、頭が動いたときに力がした仕事を報酬に足す。
//! 位置エネルギーは 引力なら -強さ * 近さ、斥力なら +強さ * 近さ で、
//! 近さは max(0, 1 - 距離 / (range + 1))。距離 range 以内のマスに出入りする移動にだけ力が働き、
//! その間は 1 マスごとに 強さ / (range + 1) ずつ変わる。
//! 仕事はアイテムやブロックの位置を止めたまま「動く前 − 動いた後」の位置エネルギーで求める。
//! 取ったアイテムは距離 0 まで近づいたものとして数え、新しく出たものは次の移動から数える。
//! アイテムやブロックは動かないので、近づいたり離れたりを繰り返しても報酬は増えない。

use crate::config::ForceConfig;
use crate::env::field::{Item, ItemType};
use crate::env::snake::Position;

/// 1種類のアイテムやブロックが頭に及ぼす力
#[derive(Debug, Clone, Copy)]
struct Force {
    /// 引力は正、斥力は負
    strength: f32,
    /// 力が届く最大のマンハッタン距離
    range: f32,
}

impl Force {
    const NONE: Force = Force {
        strength: 0.0,
        range: 0.0,
    };

    /// 距離 0 で 1、そこから 1 マスごとに 1 / (range + 1) ずつ減り、range より遠くでは 0
    fn closeness(self, source: Position, head: Position) -> f32 {
        (1.0 - source.manhattan(head) as f32 / (self.range + 1.0)).max(0.0)
    }

    fn work(self, source: Position, from: Position, to: Position) -> f32 {
        if self.strength == 0.0 {
            return 0.0;
        }
        self.strength * (self.closeness(source, to) - self.closeness(source, from))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ForceField {
    normal_apple: Force,
    gold_apple: Force,
    poison_apple: Force,
    jam_block: Force,
}

impl ForceField {
    pub fn new(cfg: &ForceConfig) -> Self {
        let attraction = |strength| Force {
            strength,
            range: cfg.attraction_range as f32,
        };
        let repulsion = |strength: f32| Force {
            strength: -strength,
            range: cfg.repulsion_range as f32,
        };
        Self {
            normal_apple: attraction(cfg.normal_apple_attraction),
            gold_apple: attraction(cfg.gold_apple_attraction),
            poison_apple: repulsion(cfg.poison_apple_repulsion),
            jam_block: repulsion(cfg.jam_block_repulsion),
        }
    }

    /// 力が1つでも働くか
    pub fn is_active(&self) -> bool {
        [
            self.normal_apple,
            self.gold_apple,
            self.poison_apple,
            self.jam_block,
        ]
        .iter()
        .any(|f| f.strength != 0.0)
    }

    fn item_force(&self, kind: ItemType) -> Force {
        match kind {
            ItemType::NormalApple => self.normal_apple,
            ItemType::GoldApple => self.gold_apple,
            ItemType::PoisonApple => self.poison_apple,
            ItemType::BlockClear | ItemType::BlockJam => Force::NONE,
        }
    }

    /// `items` と `obstacles` (お邪魔ブロック) を止めたまま頭が `from` から `to` へ動いたときに、力が頭にした仕事
    pub fn work(
        &self,
        items: &[Item],
        obstacles: &[Position],
        from: Position,
        to: Position,
    ) -> f32 {
        if from == to {
            return 0.0;
        }
        let items = items
            .iter()
            .map(|item| self.item_force(item.kind).work(item.pos, from, to));
        let blocks = obstacles
            .iter()
            .map(|&block| self.jam_block.work(block, from, to));
        items.chain(blocks).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ForceConfig {
        ForceConfig {
            attraction_range: 30,
            repulsion_range: 1,
            normal_apple_attraction: 0.1,
            gold_apple_attraction: 0.3,
            poison_apple_repulsion: 0.2,
            jam_block_repulsion: 1.0,
        }
    }

    fn field() -> ForceField {
        ForceField::new(&config())
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

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn attraction_rewards_approaching_apples() {
        let f = field();
        let apple = [item(ItemType::NormalApple, 5, 0)];
        assert!(f.work(&apple, &[], pos(0, 0), pos(1, 0)) > 0.0);
        assert!(f.work(&apple, &[], pos(1, 0), pos(0, 0)) < 0.0);
        // range 30 なら、どの 1 マスも 強さ / 31
        let eat = f.work(&apple, &[], pos(4, 0), pos(5, 0));
        assert!(close(eat, 0.1 / 31.0), "{eat}");

        let gold = [item(ItemType::GoldApple, 5, 0)];
        assert!(close(f.work(&gold, &[], pos(4, 0), pos(5, 0)), 3.0 * eat));
    }

    #[test]
    fn attraction_reaches_whole_board() {
        let f = field();
        // 16x16 の盤面の対角 (距離 30) から 1 マス近づいても、近くと同じだけ引かれる
        let apple = [item(ItemType::NormalApple, 0, 0)];
        let far = f.work(&apple, &[], pos(15, 15), pos(14, 15));
        assert!(close(far, 0.1 / 31.0), "{far}");
        // 距離 30 から取るまでの合計は 強さ * 30 / 31
        let path: Vec<Position> = (0..=15)
            .map(|x| pos(15 - x, 15))
            .chain((0..15).map(|y| pos(0, 14 - y)))
            .collect();
        let total: f32 = path
            .windows(2)
            .map(|w| f.work(&apple, &[], w[0], w[1]))
            .sum();
        assert!(close(total, 0.1 * 30.0 / 31.0), "{total}");
    }

    #[test]
    fn repulsion_acts_only_next_to_poison() {
        let f = field();
        let poison = [item(ItemType::PoisonApple, 5, 0)];
        // 2 マス以上離れていれば働かない
        assert_eq!(f.work(&poison, &[], pos(2, 0), pos(3, 0)), 0.0);
        // 隣に入る・隣から取るで 強さ / 2 ずつ引かれ、離れると戻る
        assert!(close(f.work(&poison, &[], pos(3, 0), pos(4, 0)), -0.1));
        assert!(close(f.work(&poison, &[], pos(4, 0), pos(5, 0)), -0.1));
        assert!(close(f.work(&poison, &[], pos(4, 0), pos(3, 0)), 0.1));
    }

    #[test]
    fn repulsion_acts_only_next_to_jam_blocks() {
        let f = field();
        let blocks = [pos(5, 0)];
        assert_eq!(f.work(&[], &blocks, pos(2, 0), pos(3, 0)), 0.0);
        assert!(close(f.work(&[], &blocks, pos(3, 0), pos(4, 0)), -0.5));
        assert!(close(f.work(&[], &blocks, pos(4, 0), pos(4, 1)), 0.5));
        // 横をすり抜けると、隣に入る分と離れる分で打ち消し合う
        let pass = [pos(4, 1), pos(5, 1), pos(6, 1)];
        let total: f32 = pass
            .windows(2)
            .map(|w| f.work(&[], &blocks, w[0], w[1]))
            .sum();
        assert!(close(total, 0.0), "{total}");
        // ブロックは複数あればそれぞれ働く
        let two = [pos(5, 0), pos(3, 0)];
        assert!(close(f.work(&[], &two, pos(4, 1), pos(4, 0)), -1.0));
    }

    #[test]
    fn round_trip_earns_nothing() {
        let f = field();
        let items = [
            item(ItemType::NormalApple, 2, 3),
            item(ItemType::GoldApple, 7, 1),
            item(ItemType::PoisonApple, 4, 4),
        ];
        let blocks = [pos(2, 2), pos(5, 3)];
        let path = [pos(3, 3), pos(3, 2), pos(4, 2), pos(4, 3), pos(3, 3)];
        let total: f32 = path
            .windows(2)
            .map(|w| f.work(&items, &blocks, w[0], w[1]))
            .sum();
        assert!(total.abs() < 1e-6, "{total}");
    }

    #[test]
    fn items_without_force_are_ignored() {
        let f = field();
        let items = [
            item(ItemType::BlockClear, 1, 0),
            item(ItemType::BlockJam, 1, 0),
        ];
        assert_eq!(f.work(&items, &[], pos(0, 0), pos(1, 0)), 0.0);

        let off_config = ForceConfig {
            normal_apple_attraction: 0.0,
            gold_apple_attraction: 0.0,
            poison_apple_repulsion: 0.0,
            jam_block_repulsion: 0.0,
            ..config()
        };
        let off = ForceField::new(&off_config);
        assert!(!off.is_active());
        assert_eq!(off.work(&[], &[pos(5, 0)], pos(3, 0), pos(4, 0)), 0.0);
        assert!(field().is_active());

        let blocks_only = ForceField::new(&ForceConfig {
            jam_block_repulsion: 1.0,
            ..off_config
        });
        assert!(blocks_only.is_active());
    }
}
