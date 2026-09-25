use crate::config::ModelConfig;
use crate::env::game::Action;
use crate::env::observation::{GRID_CHANNELS, VECTOR_FEATURES};
use crate::env::rules::Rules;
use tch::{nn, Tensor};

pub const KERNEL_SIZE: i64 = 3;
pub const PADDING: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvSpec {
    pub in_channels: i64,
    pub out_channels: i64,
    pub stride: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DenseSpec {
    pub in_features: i64,
    pub out_features: i64,
}

/// ネットワークの各層の形。モデル JSON の読み書きでも使う
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSpec {
    pub height: i64,
    pub width: i64,
    pub conv: Vec<ConvSpec>,
    pub dense: Vec<DenseSpec>,
}

impl NetworkSpec {
    pub fn new(model: &ModelConfig, rules: &Rules) -> Self {
        let (mut c, mut h, mut w) = (
            GRID_CHANNELS as i64,
            rules.height as i64,
            rules.width as i64,
        );
        let conv = model
            .conv_layers
            .iter()
            .map(|layer| {
                let spec = ConvSpec {
                    in_channels: c,
                    out_channels: layer.channels,
                    stride: layer.stride,
                };
                c = layer.channels;
                h = conv_out(h, layer.stride);
                w = conv_out(w, layer.stride);
                spec
            })
            .collect();

        let mut features = c * h * w + VECTOR_FEATURES as i64;
        let mut dense = Vec::new();
        for &units in model.hidden_layers.iter().chain([&(Action::COUNT as i64)]) {
            dense.push(DenseSpec {
                in_features: features,
                out_features: units,
            });
            features = units;
        }

        Self {
            height: rules.height as i64,
            width: rules.width as i64,
            conv,
            dense,
        }
    }

    pub fn parameter_count(&self) -> i64 {
        let conv: i64 = self
            .conv
            .iter()
            .map(|c| c.out_channels * (c.in_channels * KERNEL_SIZE * KERNEL_SIZE + 1))
            .sum();
        let dense: i64 = self
            .dense
            .iter()
            .map(|d| d.out_features * (d.in_features + 1))
            .sum();
        conv + dense
    }
}

pub fn conv_out(size: i64, stride: i64) -> i64 {
    (size + 2 * PADDING - KERNEL_SIZE) / stride + 1
}

/// 盤面 [B, C, H, W] と状態ベクトル [B, V] から行動ごとの Q 値 [B, A] を出す
#[derive(Debug)]
pub struct QNetwork {
    pub conv: Vec<nn::Conv2D>,
    pub dense: Vec<nn::Linear>,
}

impl QNetwork {
    pub fn new(vs: &nn::Path, spec: &NetworkSpec) -> Self {
        let conv = spec
            .conv
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let cfg = nn::ConvConfig {
                    stride: c.stride,
                    padding: PADDING,
                    ..Default::default()
                };
                nn::conv2d(
                    vs / format!("conv{i}"),
                    c.in_channels,
                    c.out_channels,
                    KERNEL_SIZE,
                    cfg,
                )
            })
            .collect();
        let dense = spec
            .dense
            .iter()
            .enumerate()
            .map(|(i, d)| {
                nn::linear(
                    vs / format!("dense{i}"),
                    d.in_features,
                    d.out_features,
                    Default::default(),
                )
            })
            .collect();
        Self { conv, dense }
    }

    pub fn forward(&self, grid: &Tensor, vector: &Tensor) -> Tensor {
        let mut x = grid.shallow_clone();
        for conv in &self.conv {
            x = x.apply(conv).relu();
        }
        let mut x = Tensor::cat(&[x.flatten(1, -1), vector.shallow_clone()], 1);
        let last = self.dense.len() - 1;
        for (i, dense) in self.dense.iter().enumerate() {
            x = x.apply(dense);
            if i < last {
                x = x.relu();
            }
        }
        x
    }
}
