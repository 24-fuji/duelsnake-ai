/**
 * 同梱の学習済みモデルの一覧。learn は盤面サイズごとに model/recent-model/snake-model-<幅>x<高さ>.json を書き出す。
 * 盤面サイズはファイル名から読み取る (中身の盤面サイズと一致することはテストで確かめている)
 */

export interface BoardSize {
  width: number;
  height: number;
}

export interface BundledModel extends BoardSize {
  /** "16x16" の形。選択肢の値に使う */
  key: string;
  /** リポジトリ直下からのパス */
  path: string;
  url: string;
}

/** ファイル名が snake-model-<幅>x<高さ>.json なら盤面サイズを返す */
export function sizeFromFileName(path: string): BoardSize | null {
  const match = /(?:^|\/)snake-model-(\d+)x(\d+)\.json$/.exec(path);
  return match ? { width: Number(match[1]), height: Number(match[2]) } : null;
}

/** パスから URL への対応を一覧にする。名前の合わないファイルは除き、小さい盤面から順に並べる */
export function bundledModels(urls: Record<string, string>): BundledModel[] {
  return Object.entries(urls)
    .flatMap(([path, url]) => {
      const size = sizeFromFileName(path);
      if (!size) return [];
      return [{ ...size, key: `${size.width}x${size.height}`, path: path.replace(/^(\.\.\/)+/, ""), url }];
    })
    .sort((a, b) => a.width * a.height - b.width * b.height || a.width - b.width);
}

export const BUNDLED_MODELS = bundledModels(
  import.meta.glob<string>("../../../model/recent-model/snake-model-*.json", {
    query: "?url",
    import: "default",
    eager: true,
  }),
);
