import { describe, expect, it } from "vitest";
import modelJson from "../../../model/recent-model/snake-model.json";
import { GameEnv, type Action } from "../game/game";
import { rulesFromConfig } from "../game/rules";
import { AiPlayer, assertModelCompatible } from "./agent";
import { SnakeModel, type ModelFile } from "./model";

const model = new SnakeModel(modelJson as unknown as ModelFile);

describe("学習済みモデル", () => {
  it("リポジトリのモデルがこの Web 版で使える", () => {
    expect(() => assertModelCompatible(model)).not.toThrow();
    expect(model.info.actions).toEqual(["up", "down", "left", "right", "boost", "use_item"]);
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
    expect(env.tick).toBeLessThanOrEqual(300);
  });
});
