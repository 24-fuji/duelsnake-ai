/** config.yaml の game セクション (モデル JSON の game) と同じ構造 */
export interface GameRules {
  grid: {
    width: number;
    height: number;
    tick_seconds: number;
    time_limit_seconds: number;
  };
  snake: {
    initial_length: number;
    normal_speed_interval_sec: number;
    boost_speed_interval_sec: number;
    boost_duration_sec: number;
    boost_cooldown_sec: number;
  };
  items: {
    apple_respawn_delay_sec: number;
    special_respawn_delay_sec: number;
    jam_block_delay_sec: number;
    jam_block_safe_distance: number;
    max_apples: number;
    max_poison: number;
    normal_apple_score: number;
    gold_apple_score: number;
    poison_apple_score: number;
    normal_apple_grow: number;
    gold_apple_grow: number;
    poison_apple_grow: number;
  };
}

/** アイテムを食べたときのスコアと長さの増減 */
export interface AppleEffect {
  score: number;
  grow: number;
}

/** ゲーム中に参照するルール値。秒数はすべてティック数に変換済み (learn/src/env/rules.rs と同じ) */
export interface Rules {
  width: number;
  height: number;
  tickSeconds: number;
  timeLimitTicks: number;
  initialLength: number;
  normalIntervalTicks: number;
  boostIntervalTicks: number;
  boostDurationTicks: number;
  boostCooldownTicks: number;
  appleRespawnTicks: number;
  specialRespawnTicks: number;
  jamDelayTicks: number;
  jamSafeDistance: number;
  normalApples: number;
  poisonApples: number;
  normalApple: AppleEffect;
  goldApple: AppleEffect;
  poisonApple: AppleEffect;
}

export function rulesFromConfig(cfg: GameRules): Rules {
  const tick = cfg.grid.tick_seconds;
  // 0.6 / 0.1 = 5.999... のような誤差で1ティックずれないよう四捨五入する
  const ticks = (sec: number) => Math.max(0, Math.round(sec / tick));
  const items = cfg.items;

  return {
    width: cfg.grid.width,
    height: cfg.grid.height,
    tickSeconds: tick,
    timeLimitTicks: Math.max(1, ticks(cfg.grid.time_limit_seconds)),
    initialLength: cfg.snake.initial_length,
    normalIntervalTicks: Math.max(1, ticks(cfg.snake.normal_speed_interval_sec)),
    boostIntervalTicks: Math.max(1, ticks(cfg.snake.boost_speed_interval_sec)),
    boostDurationTicks: Math.max(1, ticks(cfg.snake.boost_duration_sec)),
    boostCooldownTicks: ticks(cfg.snake.boost_cooldown_sec),
    appleRespawnTicks: ticks(items.apple_respawn_delay_sec),
    specialRespawnTicks: ticks(items.special_respawn_delay_sec),
    jamDelayTicks: ticks(items.jam_block_delay_sec),
    jamSafeDistance: items.jam_block_safe_distance,
    normalApples: items.max_apples,
    poisonApples: items.max_poison,
    normalApple: { score: items.normal_apple_score, grow: items.normal_apple_grow },
    goldApple: { score: items.gold_apple_score, grow: items.gold_apple_grow },
    poisonApple: { score: items.poison_apple_score, grow: items.poison_apple_grow },
  };
}
