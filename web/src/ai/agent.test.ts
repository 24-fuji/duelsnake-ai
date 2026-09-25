import { describe, expect, it } from "vitest";
import { GameEnv, type Action } from "../game/game";
import { rulesFromConfig } from "../game/rules";
import { AiPlayer, assertModelCompatible } from "./agent";
import { BUNDLED_MODELS, bundledModels, sizeFromFileName } from "./catalog";
import { SnakeModel, type ModelFile } from "./model";

const files = import.meta.glob<ModelFile>("../../../model/recent-model/snake-model-*.json", {
  eager: true,
  import: "default",
});

describe("同梱モデルの一覧", () => {
  it("ファイル名から盤面サイズを読み取る", () => {
    expect(sizeFromFileName("model/recent-model/snake-model-20x12.json")).toEqual({ width: 20, height: 12 });
    expect(sizeFromFileName("snake-model.json")).toBeNull();
    expect(sizeFromFileName("snake-model-16x16-20260925-191819.json")).toBeNull();
  });

  it("小さい盤面から順に並べ、名前の合わないファイルは除く", () => {
    const list = bundledModels({
      "../../../model/recent-model/snake-model-16x16.json": "a",
      "../../../model/recent-model/snake-model.json": "b",
      "../../../model/recent-model/snake-model-10x12.json": "c",
    });
    expect(list.map((m) => [m.key, m.path, m.url])).toEqual([
      ["10x12", "model/recent-model/snake-model-10x12.json", "c"],
      ["16x16", "model/recent-model/snake-model-16x16.json", "a"],
    ]);
  });

  it("model/recent-model/ のモデルがすべて選択肢に並ぶ", () => {
    expect(BUNDLED_MODELS.length).toBeGreaterThan(0);
    expect(BUNDLED_MODELS.map((m) => m.path).sort()).toEqual(
      Object.keys(files)
        .map((path) => path.replace(/^(\.\.\/)+/, ""))
        .sort(),
    );
  });
});

describe.each(Object.entries(files))("学習済みモデル %s", (path, file) => {
  const model = new SnakeModel(file);

  it("この Web 版で使え、ファイル名と盤面サイズが一致する", () => {
    expect(() => assertModelCompatible(model)).not.toThrow();
    expect(model.info.actions).toEqual(["up", "down", "left", "right", "boost", "use_item"]);
    const { width, height } = model.info.game.grid;
    expect(sizeFromFileName(path)).toEqual({ width, height });
  });

  it("AI 同士で最後まで対戦できる", () => {
    const env = new GameEnv(rulesFromConfig(model.info.game));
    const ais = [new AiPlayer(model, 0), new AiPlayer(model, 0)];
    while (!env.isOver) {
      const inputs: [Action[], Action[]] = [[], []];
      ais.forEach((ai, p) => {
        if (env.needsDecision(p)) {
          const decision = ai.decide(env, p);
          expect(decision.qValues.every(Number.isFinite)).toBe(true);
          inputs[p].push(decision.action);
        }
      });
      env.step(inputs);
    }
    expect(env.result).not.toBeNull();
    expect(env.tick).toBeLessThanOrEqual(env.rules.timeLimitTicks);
    // 1試合で数百回推論するので、遅い CI マシンでも既定の 5 秒で切れないようにする
  }, 30_000);
});
