import { describe, expect, it } from "vitest";
import { advanceCheatBuffer } from "./cheats";

const NAMES = ["win force", "win now"];

function type(text: string) {
  let buffer = "";
  let matched: string | null = null;
  for (const key of text) {
    ({ buffer, matched } = advanceCheatBuffer(buffer, key, NAMES));
    if (matched) return matched;
  }
  return null;
}

describe("advanceCheatBuffer", () => {
  it("コマンドをそのまま打つと一致する", () => {
    expect(type("win force")).toBe("win force");
    expect(type("win now")).toBe("win now");
  });

  it("途中で間違えても、打ち直せば一致する", () => {
    expect(type("wwin force")).toBe("win force");
    expect(type("wixwin force")).toBe("win force");
  });

  it("別の文字列では一致しない", () => {
    expect(type("win")).toBeNull();
    expect(type("wi nforce")).toBeNull();
  });
});
