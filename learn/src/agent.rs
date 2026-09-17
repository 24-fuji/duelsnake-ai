use crate::config::AgentConfig;
use tch::{nn, nn::Module, nn::OptimizerConfig, Device, Tensor};

#[derive(Debug)]
pub struct QNetwork {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
}

impl QNetwork {
    pub fn new(vs: &nn::Path, in_channels: i64, num_actions: i64) -> Self {
        let conv1 = nn::conv2d(vs, in_channels, 32, 3, nn::ConvConfig { padding: 1, ..Default::default() });
        let conv2 = nn::conv2d(vs, 32, 64, 3, nn::ConvConfig { padding: 1, ..Default::default() });
        let fc1 = nn::linear(vs, 64 * 16 * 16, 256, Default::default());
        let fc2 = nn::linear(vs, 256, num_actions, Default::default());

        Self { conv1, conv2, fc1, fc2 }
    }
}

impl Module for QNetwork {
    fn forward(&self, xs: &Tensor) -> Tensor {
        xs.apply(&self.conv1)
            .relu()
            .apply(&self.conv2)
            .relu()
            .flat_view()
            .apply(&self.fc1)
            .relu()
            .apply(&self.fc2)
    }
}

pub struct DQNAgent {
    pub q_network: QNetwork,
    pub optimizer: nn::Optimizer,
    pub vs: nn::VarStore,
}

impl DQNAgent {
    pub fn new(config: &AgentConfig) -> Self {
        let vs = nn::VarStore::new(Device::Cpu);
        let q_network = QNetwork::new(&vs.root(), config.input_channels, config.num_actions);
        let optimizer = nn::Adam::default().build(&vs, config.learning_rate).unwrap();

        Self { q_network, optimizer, vs }
    }

    pub fn select_action(&self, state: &Tensor, epsilon: f64, action_bound: i64) -> i64 {
        if rand::random::<f64>() < epsilon {
            rand::random::<i64>().rem_euclid(action_bound)
        } else {
            let state_batch = state.unsqueeze(0);
            let q_values = self.q_network.forward(&state_batch);
            q_values.argmax(-1, false).int64_value(&[0])
        }
    }
}