//! 学習済みモデルの JSON 形式。Web からそのまま読み込めるよう、構造・ルール・重みを1ファイルにまとめる

use crate::config::{Config, DifficultyConfig, GameConfig, GridConfig};
use crate::env::game::Action;
use crate::env::observation::{
    GRID_CHANNEL_NAMES, INCOMING_JAMS_SCALE, PENDING_GROWTH_SCALE, SCORE_DIFF_SCALE,
    VECTOR_FEATURE_NAMES,
};
use crate::model::{NetworkSpec, QNetwork, KERNEL_SIZE, PADDING};
use serde::ser::Error as _;
use serde::{Deserialize, Serialize, Serializer};
use std::fs;
use std::path::Path;
use tch::Tensor;

pub const FORMAT: &str = "duelsnake-dqn";
pub const FORMAT_VERSION: u32 = 1;

/// ファイル名に付ける盤面サイズ (例: 16x16)
pub fn size_label(grid: &GridConfig) -> String {
    format!("{}x{}", grid.width, grid.height)
}

/// 盤面サイズごとの最新モデルのファイル名 (例: snake-model-16x16.json)。Web 側はこの名前から盤面サイズを読み取る
pub fn model_file_name(grid: &GridConfig) -> String {
    format!("snake-model-{}.json", size_label(grid))
}

/// `model_file_name` の形のファイル名なら、その盤面サイズの部分を返す
pub fn size_from_file_name(name: &str) -> Option<&str> {
    let size = name.strip_prefix("snake-model-")?.strip_suffix(".json")?;
    let (w, h) = size.split_once('x')?;
    let is_number = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    (is_number(w) && is_number(h)).then_some(size)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelFile {
    pub format: String,
    pub format_version: u32,
    pub created_at: String,
    pub training: TrainingInfo,
    pub game: GameConfig,
    pub observation: ObservationSpec,
    pub actions: Vec<String>,
    pub difficulty: Vec<DifficultyConfig>,
    pub network: NetworkWeights,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TrainingInfo {
    pub games: u64,
    pub decisions: u64,
    pub updates: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObservationSpec {
    pub grid_channels: Vec<String>,
    pub grid_height: i64,
    pub grid_width: i64,
    pub vector_features: Vec<String>,
    pub pending_growth_scale: f32,
    pub score_diff_scale: f32,
    pub incoming_jams_scale: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkWeights {
    pub conv: Vec<ConvWeights>,
    pub dense: Vec<DenseWeights>,
}

/// weight は [out_channels][in_channels][kernel_size][kernel_size] を平坦化したもの
#[derive(Debug, Serialize, Deserialize)]
pub struct ConvWeights {
    pub in_channels: i64,
    pub out_channels: i64,
    pub kernel_size: i64,
    pub stride: i64,
    pub padding: i64,
    #[serde(serialize_with = "inline_array")]
    pub weight: Vec<f32>,
    #[serde(serialize_with = "inline_array")]
    pub bias: Vec<f32>,
}

/// weight は [out_features][in_features] を平坦化したもの
#[derive(Debug, Serialize, Deserialize)]
pub struct DenseWeights {
    pub in_features: i64,
    pub out_features: i64,
    #[serde(serialize_with = "inline_array")]
    pub weight: Vec<f32>,
    #[serde(serialize_with = "inline_array")]
    pub bias: Vec<f32>,
}

/// 重み配列は整形出力でも1行にまとめる (1要素1行だと数百万行になるため)
fn inline_array<S: Serializer>(values: &[f32], serializer: S) -> Result<S::Ok, S::Error> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(S::Error::custom("重みに NaN または無限大が含まれています"));
    }
    let json = serde_json::to_string(values).map_err(S::Error::custom)?;
    serde_json::value::RawValue::from_string(json)
        .map_err(S::Error::custom)?
        .serialize(serializer)
}

fn to_vec(t: &Tensor) -> Vec<f32> {
    Vec::<f32>::try_from(t.flatten(0, -1)).expect("f32 テンソルへの変換に失敗")
}

impl ModelFile {
    pub fn new(
        config: &Config,
        net: &QNetwork,
        spec: &NetworkSpec,
        training: TrainingInfo,
    ) -> Self {
        let conv = net
            .conv
            .iter()
            .zip(&spec.conv)
            .map(|(layer, s)| ConvWeights {
                in_channels: s.in_channels,
                out_channels: s.out_channels,
                kernel_size: KERNEL_SIZE,
                stride: s.stride,
                padding: PADDING,
                weight: to_vec(&layer.ws),
                bias: to_vec(layer.bs.as_ref().expect("畳み込み層にバイアスがありません")),
            })
            .collect();
        let dense = net
            .dense
            .iter()
            .zip(&spec.dense)
            .map(|(layer, s)| DenseWeights {
                in_features: s.in_features,
                out_features: s.out_features,
                weight: to_vec(&layer.ws),
                bias: to_vec(layer.bs.as_ref().expect("全結合層にバイアスがありません")),
            })
            .collect();

        Self {
            format: FORMAT.to_string(),
            format_version: FORMAT_VERSION,
            created_at: chrono::Local::now().to_rfc3339(),
            training,
            game: config.game.clone(),
            observation: ObservationSpec {
                grid_channels: GRID_CHANNEL_NAMES.iter().map(|s| s.to_string()).collect(),
                grid_height: spec.height,
                grid_width: spec.width,
                vector_features: VECTOR_FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
                pending_growth_scale: PENDING_GROWTH_SCALE,
                score_diff_scale: SCORE_DIFF_SCALE,
                incoming_jams_scale: INCOMING_JAMS_SCALE,
            },
            actions: Action::ALL.iter().map(|a| a.name().to_string()).collect(),
            difficulty: config.export.difficulty.clone(),
            network: NetworkWeights { conv, dense },
        }
    }

    pub fn read(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("モデル {} を読めません: {e}", path.display()))?;
        let model: ModelFile = serde_json::from_str(&content)
            .map_err(|e| format!("モデル {} の形式が不正です: {e}", path.display()))?;
        if model.format != FORMAT || model.format_version != FORMAT_VERSION {
            return Err(format!(
                "未対応のモデル形式です: {} v{}",
                model.format, model.format_version
            )
            .into());
        }
        Ok(model)
    }

    /// 一時ファイルに書いてから置き換えるので、書き込み中に中断しても壊れたファイルが残らない
    pub fn write(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }

    /// 現在の設定・観測仕様と互換性があるか確認し、重みをネットワークに読み込む
    pub fn load_into(&self, net: &mut QNetwork, spec: &NetworkSpec) -> Result<(), String> {
        let same_names = |names: &[String], expected: &[&str]| {
            names
                .iter()
                .map(String::as_str)
                .eq(expected.iter().copied())
        };
        if !same_names(&self.observation.grid_channels, &GRID_CHANNEL_NAMES)
            || !same_names(&self.observation.vector_features, &VECTOR_FEATURE_NAMES)
        {
            return Err("モデルの観測仕様が現在のコードと一致しません".into());
        }
        let shapes_match =
            self.network.conv.len() == spec.conv.len()
                && self.network.dense.len() == spec.dense.len()
                && self.network.conv.iter().zip(&spec.conv).all(|(w, s)| {
                    (
                        w.in_channels,
                        w.out_channels,
                        w.stride,
                        w.kernel_size,
                        w.padding,
                    ) == (
                        s.in_channels,
                        s.out_channels,
                        s.stride,
                        KERNEL_SIZE,
                        PADDING,
                    )
                })
                && self.network.dense.iter().zip(&spec.dense).all(|(w, s)| {
                    (w.in_features, w.out_features) == (s.in_features, s.out_features)
                });
        if !shapes_match {
            return Err("モデルの層構成が config.yaml の model / grid 設定と一致しません".into());
        }

        let copy = |dst: &mut Tensor, src: &[f32]| -> Result<(), String> {
            if dst.numel() != src.len() {
                return Err(format!(
                    "重みの要素数が一致しません ({} != {})",
                    src.len(),
                    dst.numel()
                ));
            }
            let value = Tensor::from_slice(src)
                .view(dst.size().as_slice())
                .to(dst.device());
            tch::no_grad(|| dst.copy_(&value));
            Ok(())
        };
        for (layer, w) in net.conv.iter_mut().zip(&self.network.conv) {
            copy(&mut layer.ws, &w.weight)?;
            copy(layer.bs.as_mut().unwrap(), &w.bias)?;
        }
        for (layer, w) in net.dense.iter_mut().zip(&self.network.dense) {
            copy(&mut layer.ws, &w.weight)?;
            copy(layer.bs.as_mut().unwrap(), &w.bias)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::observation::{GRID_CHANNELS, VECTOR_FEATURES};
    use crate::env::rules::Rules;
    use tch::{nn, Device, Kind};

    fn config() -> Config {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.yaml");
        Config::load(&path).unwrap()
    }

    fn build(config: &Config) -> (nn::VarStore, QNetwork, NetworkSpec) {
        let spec = NetworkSpec::new(&config.model, &Rules::from_config(&config.game));
        let vs = nn::VarStore::new(Device::Cpu);
        let net = QNetwork::new(&vs.root(), &spec);
        (vs, net, spec)
    }

    /// Web 側 (web/src/ai/model.ts) と同じ手順の素朴な順伝播
    fn reference_forward(
        net: &NetworkWeights,
        height: i64,
        width: i64,
        grid: &[f32],
        vector: &[f32],
    ) -> Vec<f32> {
        let (mut x, mut h, mut w) = (grid.to_vec(), height as usize, width as usize);
        for layer in &net.conv {
            let (cin, cout) = (layer.in_channels as usize, layer.out_channels as usize);
            let (k, s, p) = (
                layer.kernel_size as usize,
                layer.stride as usize,
                layer.padding as usize,
            );
            let (oh, ow) = ((h + 2 * p - k) / s + 1, (w + 2 * p - k) / s + 1);
            let mut out = vec![0.0f32; cout * oh * ow];
            for oc in 0..cout {
                for oy in 0..oh {
                    for ox in 0..ow {
                        let mut sum = layer.bias[oc];
                        for ic in 0..cin {
                            for ky in 0..k {
                                let iy = (oy * s + ky) as isize - p as isize;
                                if iy < 0 || iy >= h as isize {
                                    continue;
                                }
                                for kx in 0..k {
                                    let ix = (ox * s + kx) as isize - p as isize;
                                    if ix < 0 || ix >= w as isize {
                                        continue;
                                    }
                                    sum += layer.weight[((oc * cin + ic) * k + ky) * k + kx]
                                        * x[(ic * h + iy as usize) * w + ix as usize];
                                }
                            }
                        }
                        out[(oc * oh + oy) * ow + ox] = sum.max(0.0);
                    }
                }
            }
            (x, h, w) = (out, oh, ow);
        }
        x.extend_from_slice(vector);
        let last = net.dense.len() - 1;
        for (i, layer) in net.dense.iter().enumerate() {
            let n_in = layer.in_features as usize;
            let mut out = layer.bias.clone();
            for (o, value) in out.iter_mut().enumerate() {
                *value += (0..n_in)
                    .map(|j| layer.weight[o * n_in + j] * x[j])
                    .sum::<f32>();
                if i < last {
                    *value = value.max(0.0);
                }
            }
            x = out;
        }
        x
    }

    fn random_inputs(spec: &NetworkSpec, batch: i64) -> (Tensor, Tensor) {
        let grid = Tensor::rand(
            [batch, GRID_CHANNELS as i64, spec.height, spec.width],
            (Kind::Float, Device::Cpu),
        );
        let vector = Tensor::rand([batch, VECTOR_FEATURES as i64], (Kind::Float, Device::Cpu));
        (grid, vector)
    }

    #[test]
    fn json_round_trip_reproduces_outputs_exactly() {
        tch::manual_seed(1);
        let config = config();
        let (_vs, net, spec) = build(&config);
        let json = serde_json::to_string_pretty(&ModelFile::new(
            &config,
            &net,
            &spec,
            TrainingInfo::default(),
        ))
        .unwrap();

        let loaded: ModelFile = serde_json::from_str(&json).unwrap();
        let (_vs2, mut net2, _) = build(&config);
        loaded.load_into(&mut net2, &spec).unwrap();

        let (grid, vector) = random_inputs(&spec, 8);
        let (a, b) = tch::no_grad(|| (net.forward(&grid, &vector), net2.forward(&grid, &vector)));
        assert_eq!(
            Vec::<f32>::try_from(a.flatten(0, -1)).unwrap(),
            Vec::<f32>::try_from(b.flatten(0, -1)).unwrap()
        );
    }

    #[test]
    fn reference_forward_matches_tch() {
        tch::manual_seed(2);
        let config = config();
        let (_vs, net, spec) = build(&config);
        let model = ModelFile::new(&config, &net, &spec, TrainingInfo::default());

        let (grid, vector) = random_inputs(&spec, 1);
        let expected =
            Vec::<f32>::try_from(tch::no_grad(|| net.forward(&grid, &vector)).flatten(0, -1))
                .unwrap();
        let actual = reference_forward(
            &model.network,
            model.observation.grid_height,
            model.observation.grid_width,
            &Vec::<f32>::try_from(grid.flatten(0, -1)).unwrap(),
            &Vec::<f32>::try_from(vector.flatten(0, -1)).unwrap(),
        );
        assert_eq!(actual.len(), Action::COUNT);
        for (a, e) in actual.iter().zip(&expected) {
            assert!((a - e).abs() < 1e-4, "{actual:?} != {expected:?}");
        }
    }

    #[test]
    fn file_name_carries_grid_size() {
        let mut grid = config().game.grid;
        (grid.width, grid.height) = (20, 12);
        let name = model_file_name(&grid);
        assert_eq!(name, "snake-model-20x12.json");
        assert_eq!(size_from_file_name(&name), Some("20x12"));
        for other in [
            "snake-model.json",
            "snake-model-16x16-20260925-191819.json",
            "snake-model-x16.json",
            "snake-model-16x16.json.tmp",
        ] {
            assert_eq!(size_from_file_name(other), None, "{other}");
        }
    }

    #[test]
    fn mismatched_architecture_is_rejected() {
        let config = config();
        let (_vs, net, spec) = build(&config);
        let model = ModelFile::new(&config, &net, &spec, TrainingInfo::default());

        let mut other = config.clone();
        other.model.hidden_layers.push(32);
        let (_vs2, mut net2, spec2) = build(&other);
        assert!(model.load_into(&mut net2, &spec2).is_err());
    }
}
