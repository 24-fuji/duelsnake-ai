import { describe, expect, it } from "vitest";
import modelFile from "../../../model/recent-model/snake-model-16x16.json";
import { GameEnv, type Action } from "../game/game";
import { rulesFromConfig, type GameRules } from "../game/rules";
import { assistLevel, collisionAhead } from "./assist";

function readyEnv(): GameEnv {
  const env = new GameEnv(rulesFromConfig(modelFile.game as GameRules));
  for (const field of env.fields) field.items = [];
  while (!env.needsDecision(0)) env.step([[], []]);
  return env;
}

describe("assistLevel", () => {
  it("難易度ごとに補助の範囲が決まる", () => {
    expect(assistLevel("easy")).toBe("all");
    expect(assistLevel("medium")).toBe("wall");
    expect(assistLevel("hard")).toBe("none");
  });
});

describe("collisionAhead", () => {
  const head = (env: GameEnv, x: number, y: number, dir: "up" | "left") => {
    const snake = env.fields[0].snake;
    snake.body = [{ x, y }];
    snake.dir = dir;
    snake.lastMovedDir = dir;
  };

  it("外周の壁は ふつう・かんたん で対象になる", () => {
    const env = readyEnv();
    head(env, 5, 0, "up");
    expect(collisionAhead(env, 0, [], "wall")).toBe(true);
    expect(collisionAhead(env, 0, [], "all")).toBe(true);
    expect(collisionAhead(env, 0, [], "none")).toBe(false);
    expect(collisionAhead(env, 0, ["left" as Action], "wall")).toBe(false);
  });

  it("お邪魔ブロックは かんたん だけ対象になる", () => {
    const env = readyEnv();
    head(env, 5, 5, "left");
    env.fields[0].obstacles = [{ x: 4, y: 5 }];
    expect(collisionAhead(env, 0, [], "wall")).toBe(false);
    expect(collisionAhead(env, 0, [], "all")).toBe(true);
  });

  it("自分の体は かんたん だけ対象になる", () => {
    const env = readyEnv();
    const snake = env.fields[0].snake;
    snake.body = [{ x: 5, y: 5 }, { x: 5, y: 6 }, { x: 4, y: 6 }, { x: 4, y: 5 }, { x: 4, y: 4 }];
    snake.dir = "left";
    snake.lastMovedDir = "up";
    expect(collisionAhead(env, 0, [], "wall")).toBe(false);
    expect(collisionAhead(env, 0, [], "all")).toBe(true);
  });
});
