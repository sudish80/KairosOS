use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct KinematicsEngine {
    config: Arc<RwLock<config::Config>>,
}
impl KinematicsEngine {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub fn forward(&self, _joint_angles: &[f64]) -> Vec<f64> {
        vec![0.0; 3]
    }
    pub fn inverse(&self, _target: &[f64]) -> Vec<f64> {
        vec![0.0; self.config.blocking_read().kinematics.num_joints as usize]
    }
}
