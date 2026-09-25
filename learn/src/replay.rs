use rand::rngs::StdRng;
use rand::Rng;
use std::collections::VecDeque;
use tch::{Device, Kind, Tensor};

/// リプレイに保存する観測。盤面はメモリ節約のため 0..=255 に量子化する
#[derive(Debug, Clone)]
pub struct Obs {
    pub grid: Vec<u8>,
    pub vector: Vec<f32>,
}

impl Obs {
    pub fn quantize(grid: &[f32], vector: &[f32]) -> Self {
        Self {
            grid: grid.iter().map(|&v| (v * 255.0).round() as u8).collect(),
            vector: vector.to_vec(),
        }
    }
}

pub struct Batch {
    pub grid: Tensor,
    pub vector: Tensor,
    pub actions: Tensor,
    pub returns: Tensor,
    pub next_grid: Tensor,
    pub next_vector: Tensor,
    pub dones: Tensor,
}

/// n ステップ遷移を貯めるリングバッファ
pub struct ReplayBuffer {
    capacity: usize,
    grid_len: usize,
    vector_len: usize,
    grid_shape: [i64; 3],
    grids: Vec<u8>,
    next_grids: Vec<u8>,
    vectors: Vec<f32>,
    next_vectors: Vec<f32>,
    actions: Vec<i64>,
    returns: Vec<f32>,
    dones: Vec<f32>,
    len: usize,
    pos: usize,
}

impl ReplayBuffer {
    pub fn new(capacity: usize, grid_shape: [i64; 3], vector_len: usize) -> Self {
        let grid_len = grid_shape.iter().product::<i64>() as usize;
        Self {
            capacity,
            grid_len,
            vector_len,
            grid_shape,
            grids: vec![0; capacity * grid_len],
            next_grids: vec![0; capacity * grid_len],
            vectors: vec![0.0; capacity * vector_len],
            next_vectors: vec![0.0; capacity * vector_len],
            actions: vec![0; capacity],
            returns: vec![0.0; capacity],
            dones: vec![0.0; capacity],
            len: 0,
            pos: 0,
        }
    }

    pub fn bytes_per_transition(grid_len: usize, vector_len: usize) -> usize {
        2 * (grid_len + vector_len * 4) + 8 + 4 + 4
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// `next` が None なら終端遷移
    pub fn push(&mut self, obs: &Obs, action: i64, ret: f32, next: Option<&Obs>) {
        let i = self.pos;
        let (g, v) = (self.grid_len, self.vector_len);
        self.grids[i * g..(i + 1) * g].copy_from_slice(&obs.grid);
        self.vectors[i * v..(i + 1) * v].copy_from_slice(&obs.vector);
        match next {
            Some(next) => {
                self.next_grids[i * g..(i + 1) * g].copy_from_slice(&next.grid);
                self.next_vectors[i * v..(i + 1) * v].copy_from_slice(&next.vector);
                self.dones[i] = 0.0;
            }
            None => {
                // 終端では次状態を使わないが、古い値が残らないよう消しておく
                self.next_grids[i * g..(i + 1) * g].fill(0);
                self.next_vectors[i * v..(i + 1) * v].fill(0.0);
                self.dones[i] = 1.0;
            }
        }
        self.actions[i] = action;
        self.returns[i] = ret;
        self.pos = (self.pos + 1) % self.capacity;
        self.len = (self.len + 1).min(self.capacity);
    }

    pub fn sample(&self, batch_size: usize, rng: &mut StdRng, device: Device) -> Batch {
        let (g, v) = (self.grid_len, self.vector_len);
        let mut grids = Vec::with_capacity(batch_size * g);
        let mut next_grids = Vec::with_capacity(batch_size * g);
        let mut vectors = Vec::with_capacity(batch_size * v);
        let mut next_vectors = Vec::with_capacity(batch_size * v);
        let mut actions = Vec::with_capacity(batch_size);
        let mut returns = Vec::with_capacity(batch_size);
        let mut dones = Vec::with_capacity(batch_size);

        for _ in 0..batch_size {
            let i = rng.gen_range(0..self.len);
            grids.extend_from_slice(&self.grids[i * g..(i + 1) * g]);
            next_grids.extend_from_slice(&self.next_grids[i * g..(i + 1) * g]);
            vectors.extend_from_slice(&self.vectors[i * v..(i + 1) * v]);
            next_vectors.extend_from_slice(&self.next_vectors[i * v..(i + 1) * v]);
            actions.push(self.actions[i]);
            returns.push(self.returns[i]);
            dones.push(self.dones[i]);
        }

        let b = batch_size as i64;
        let [c, h, w] = self.grid_shape;
        let grid_tensor = |data: &[u8]| {
            (Tensor::from_slice(data)
                .view([b, c, h, w])
                .to_kind(Kind::Float)
                / 255.0)
                .to(device)
        };
        let vector_tensor = |data: &[f32]| Tensor::from_slice(data).view([b, v as i64]).to(device);
        Batch {
            grid: grid_tensor(&grids),
            vector: vector_tensor(&vectors),
            actions: Tensor::from_slice(&actions).view([b, 1]).to(device),
            returns: Tensor::from_slice(&returns).to(device),
            next_grid: grid_tensor(&next_grids),
            next_vector: vector_tensor(&next_vectors),
            dones: Tensor::from_slice(&dones).to(device),
        }
    }
}

/// 1プレイヤー分の判断の流れから n ステップ遷移を組み立てる
pub struct NStepBuilder {
    n: usize,
    gamma: f32,
    steps: VecDeque<(Obs, i64, f32)>,
}

impl NStepBuilder {
    pub fn new(n: usize, gamma: f32) -> Self {
        Self {
            n,
            gamma,
            steps: VecDeque::with_capacity(n),
        }
    }

    fn discounted_return(&self) -> f32 {
        self.steps
            .iter()
            .rev()
            .fold(0.0, |acc, (_, _, r)| r + self.gamma * acc)
    }

    /// 1判断分の (観測, 行動, 報酬) を追加し、確定した遷移をリプレイに入れる。
    /// `next` が None なら試合終了として残りをすべて終端遷移にする。戻り値は追加した遷移数
    pub fn push(
        &mut self,
        obs: Obs,
        action: i64,
        reward: f32,
        next: Option<&Obs>,
        replay: &mut ReplayBuffer,
    ) -> usize {
        self.steps.push_back((obs, action, reward));
        match next {
            Some(next) => {
                if self.steps.len() < self.n {
                    return 0;
                }
                let ret = self.discounted_return();
                let (obs, action, _) = self.steps.pop_front().unwrap();
                replay.push(&obs, action, ret, Some(next));
                1
            }
            None => {
                let mut added = 0;
                while !self.steps.is_empty() {
                    let ret = self.discounted_return();
                    let (obs, action, _) = self.steps.pop_front().unwrap();
                    replay.push(&obs, action, ret, None);
                    added += 1;
                }
                added
            }
        }
    }
}
