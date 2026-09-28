import { useMemo } from "react";
import { rulesFromConfig, type GameRules } from "../game/rules";
import { SOLO_POISON_GROW, SOLO_POISON_MULTIPLIER, SOLO_TIME_LIMITS, soloRules } from "../game/solo";
import { itemImageUrl, type Sprite } from "./draw";
import { formatTimeLimit } from "./SoloScreen";

interface Props {
  /** 選んでいる盤面のモデルのルール。数値はここから出す */
  game: GameRules;
  onBack: () => void;
}

/** 遊ぶ人向けのルール説明。learn/RULES.md から学習や実装の話を除き、選んでいる盤面の数値で書く */
export function RulesScreen({ game, onBack }: Props) {
  const rules = useMemo(() => rulesFromConfig(game), [game]);
  const solo = useMemo(() => soloRules(rules), [rules]);
  /** ティック数を秒にする。0.30000000000000004 のような誤差は丸める */
  const sec = (ticks: number) => `${Number((ticks * rules.tickSeconds).toFixed(2))} 秒`;
  const signed = (n: number) => (n > 0 ? `+${n}` : `${n}`);
  const icon = (kind: Sprite) => <img className="item-icon" src={itemImageUrl(kind)} alt="" />;
  const maxSoloLimit = formatTimeLimit(SOLO_TIME_LIMITS[SOLO_TIME_LIMITS.length - 1]);

  return (
    <div className="rules-page">
      <div className="rules-top">
        <button onClick={onBack}>← 戻る</button>
        <h2>ルール</h2>
      </div>
      <p className="rules-note">
        盤面 {rules.width} × {rules.height} のルールです。数値は選んでいる盤面によって変わります。
      </p>

      <section>
        <h3>概要</h3>
        <ul>
          <li>2人対戦のスネークゲームです。それぞれ自分専用の盤面で、自分のヘビを操作します。</li>
          <li>2つの盤面は別々で、相手に影響を与えられるのは「お邪魔」アイテムだけです。</li>
          <li>制限時間は {sec(rules.timeLimitTicks)} です。</li>
          <li>壁・自分の体・お邪魔ブロックにぶつかると、その時点で負けです。</li>
          <li>お邪魔ブロック以外のすべてのマスを自分のヘビで埋めると、その時点で勝ちです。</li>
        </ul>
      </section>

      <section>
        <h3>ヘビの動き</h3>
        <ul>
          <li>最初は長さ {rules.initialLength} で、盤面の真ん中から上向きに始まります。</li>
          <li>
            {sec(rules.normalIntervalTicks)}ごとに 1 マス進みます。ブースト中は {sec(rules.boostIntervalTicks)}
            ごとに進みます。
          </li>
          <li>向きは次に進むまでに何度でも変えられ、最後に入力した向きへ進みます。</li>
          <li>
            直前に進んだ向きの真逆には曲がれません。上に進んでいる最中に「左 → 下」と素早く入力しても下は無視されるので、反転して自滅することはありません。
          </li>
          <li>伸びている途中でなければ、今の尻尾のマスには進めます (尻尾も同時に動くため)。</li>
        </ul>
      </section>

      <section>
        <h3>操作</h3>
        <table className="rules-table">
          <thead>
            <tr>
              <th>行動</th>
              <th>パソコン版</th>
              <th>携帯版</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>向きを変える</td>
              <td>矢印キー / WASD</td>
              <td>盤面をスワイプ</td>
            </tr>
            <tr>
              <td>ブースト</td>
              <td>スペース</td>
              <td>盤面を長押し</td>
            </tr>
            <tr>
              <td>アイテムを使う</td>
              <td>Shift / E</td>
              <td>盤面をダブルタップ</td>
            </tr>
            <tr>
              <td>開始・もう一度</td>
              <td>Enter</td>
              <td>盤面をタップ</td>
            </tr>
            <tr>
              <td>一時停止</td>
              <td>P / Esc</td>
              <td>「一時停止」ボタン (再開は盤面をタップ)</td>
            </tr>
          </tbody>
        </table>
      </section>

      <section>
        <h3>ブースト</h3>
        <ul>
          <li>使うと {sec(rules.boostDurationTicks)}の間、速く進みます。</li>
          <li>ブーストが終わってから {sec(rules.boostCooldownTicks)}は、もう一度使えません。</li>
        </ul>
      </section>

      <section>
        <h3>アイテム</h3>
        <table className="rules-table">
          <thead>
            <tr>
              <th>アイテム</th>
              <th>盤面の数</th>
              <th>取ったとき</th>
              <th>次が出るまで</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                {icon("normal_apple")} リンゴ
              </td>
              <td className="num">{rules.normalApples}</td>
              <td>
                スコア {signed(rules.normalApple.score)}、長さ {signed(rules.normalApple.grow)}
              </td>
              <td>{sec(rules.appleRespawnTicks)}</td>
            </tr>
            <tr>
              <td>
                {icon("gold_apple")} 金のリンゴ
              </td>
              <td className="num">1</td>
              <td>
                スコア {signed(rules.goldApple.score)}、長さ {signed(rules.goldApple.grow)}
              </td>
              <td>{sec(rules.specialRespawnTicks)}</td>
            </tr>
            <tr>
              <td>
                {icon("poison_apple")} 毒リンゴ
              </td>
              <td className="num">{rules.poisonApples}</td>
              <td>
                スコア {signed(rules.poisonApple.score)}、長さ {signed(rules.poisonApple.grow)}
              </td>
              <td>{sec(rules.appleRespawnTicks)}</td>
            </tr>
            <tr>
              <td>
                {icon("block_clear")} ブロック消去
              </td>
              <td className="num">1</td>
              <td>持っておき、あとで使う</td>
              <td>{sec(rules.specialRespawnTicks)}</td>
            </tr>
            <tr>
              <td>
                {icon("block_jam")} お邪魔
              </td>
              <td className="num">1</td>
              <td>持っておき、あとで使う</td>
              <td>{sec(rules.specialRespawnTicks)}</td>
            </tr>
          </tbody>
        </table>
        <ul>
          <li>頭がそのマスに入ると取れます。</li>
          <li>取ったアイテムは、少し経つと同じ種類のものが空いているマスのどこかに出ます。</li>
          <li>スコアは 0 より下がりません。</li>
          <li>伸びるとき: 取った次の移動から、1 回進むごとに 1 マスずつ伸びます。</li>
          <li>縮むとき: これから伸びる分を先に打ち消し、足りない分だけ尻尾がすぐに縮みます。長さは 1 より短くなりません。</li>
        </ul>
        <h4>持っておくアイテム</h4>
        <ul>
          <li>持てるのは 1 つだけです。持っているときに別のものを取ると、新しいほうに入れ替わります。</li>
          <li>
            {icon("block_clear")} ブロック消去: 使うと、自分の盤面のお邪魔ブロック {icon("obstacle")} をすべて消します。
          </li>
          <li>
            {icon("block_jam")} お邪魔: 使うと {sec(rules.jamDelayTicks)}後に、相手の盤面の空いているマスのどこかにお邪魔ブロックを
            1 つ置きます。相手の頭から {rules.jamSafeDistance} マス以内 (縦と横のマス数の合計) には置きません。置ける場所が無ければ、ブロックは消えます。
          </li>
          <li>お邪魔ブロックは、ブロック消去で消されるまで残ります。</li>
        </ul>
      </section>

      <section>
        <h3>勝ち負け</h3>
        <p>上から順に調べ、最初に当てはまったもので決まります。</p>
        <table className="rules-table">
          <thead>
            <tr>
              <th>状況</th>
              <th>結果</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>片方だけが盤面を埋めた</td>
              <td>埋めた側の勝ち (相手の状態やスコアは関係ない)</td>
            </tr>
            <tr>
              <td>両方が同時に盤面を埋めた</td>
              <td>スコアが高いほうの勝ち。同点なら引き分け</td>
            </tr>
            <tr>
              <td>片方だけがぶつかった</td>
              <td>ぶつかった側の負け (スコアは関係ない)</td>
            </tr>
            <tr>
              <td>両方が同時にぶつかった</td>
              <td>スコアが高いほうの勝ち。同点なら引き分け</td>
            </tr>
            <tr>
              <td>制限時間まで両方とも無事</td>
              <td>スコアが高いほうの勝ち。同点なら引き分け</td>
            </tr>
          </tbody>
        </table>
        <ul>
          <li>「盤面を埋めた」とは、自分のヘビの体とお邪魔ブロックで、盤面のすべてのマスがふさがった状態です。</li>
          <li>伸びている途中で最後の空きマスに入ると、尻尾が縮まないので盤面が埋まります。伸びている途中でなければ尻尾のマスが空くので、埋まりません。</li>
          <li>埋めた時点で勝ちが決まるので、そのあとぶつかって負けになることはありません。</li>
        </ul>
      </section>

      <section>
        <h3>一人モード</h3>
        <ul>
          <li>AI の相手がいないモードです。ブロック消去とお邪魔は出ません。</li>
          <li>
            毒リンゴが対戦の {SOLO_POISON_MULTIPLIER} 倍 ({solo.poisonApples} 個) 出て、取ると長さが {-SOLO_POISON_GROW}{" "}
            マス縮みます。
          </li>
          <li>制限時間は、なし、または 30 秒から {maxSoloLimit}まで (30 秒刻み) から選べます。</li>
          <li>すべてのマスを埋めるとクリアで、クリアまでの時間と取ったリンゴの数が表示されます。</li>
          <li>ぶつかるか、制限時間になると終わりです。それ以外は対戦と同じルールです。</li>
        </ul>
      </section>

      <div className="rules-bottom">
        <button onClick={onBack}>← 戻る</button>
      </div>
    </div>
  );
}
