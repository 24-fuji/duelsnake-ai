import { describe, expect, it } from "vitest";
import { advanceCheatBuffer } from "./cheats";

const NAMES = ["WIN FORCE", "win now"];

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
    expect(type("WIN FORCE")).toBe("WIN FORCE");
    expect(type("win now")).toBe("win now");
  });

  it("途中で間違えても、打ち直せば一致する", () => {
    expect(type("wWIN FORCE")).toBe("WIN FORCE");
    expect(type("wixWIN FORCE")).toBe("WIN FORCE");
  });

  it("別の文字列では一致しない", () => {
    expect(type("WIN")).toBeNull();
    expect(type("win force")).toBeNull();
  });
});
