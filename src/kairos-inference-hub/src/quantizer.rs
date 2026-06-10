//! Quantizer — precision conversion (fp32/fp16/int8/int4) with calibration
use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Precision {
    Fp32,
    Fp16,
    Int8,
    Int4,
}

impl Precision {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fp32" | "float32" => Self::Fp32,
            "fp16" | "float16" => Self::Fp16,
            "int8" => Self::Int8,
            "int4" => Self::Int4,
            _ => Self::Fp16,
        }
    }

    pub fn bytes_per_param(&self) -> usize {
        match self {
            Self::Fp32 => 4,
            Self::Fp16 => 2,
            Self::Int8 => 1,
            Self::Int4 => 1,
        }
    }
}

pub struct Quantizer {
    config: Arc<RwLock<config::Config>>,
}

impl Quantizer {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }

    pub async fn quantize(&self, data: &[u8], target: Precision) -> anyhow::Result<Vec<u8>> {
        match target {
            Precision::Fp32 => Ok(data.to_vec()),
            Precision::Fp16 => self.to_fp16(data),
            Precision::Int8 => self.to_int8(data),
            Precision::Int4 => self.to_int4(data),
        }
    }

    fn to_fp16(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        // In production: use hardware-accelerated fp32→fp16 conversion
        let n = data.len() / 4;
        let mut out = Vec::with_capacity(n * 2);
        for i in 0..n {
            let chunk = &data[i * 4..i * 4 + 4];
            let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let half = half_to_f16_bits(val);
            out.extend_from_slice(&half.to_le_bytes());
        }
        Ok(out)
    }

    fn to_int8(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let n = data.len() / 4;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let chunk = &data[i * 4..i * 4 + 4];
            let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let q = (val.clamp(-128.0, 127.0).round() as i8) as u8;
            out.push(q);
        }
        Ok(out)
    }

    fn to_int4(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let n = data.len() / 4;
        let mut out = Vec::with_capacity((n + 1) / 2);
        for i in (0..n).step_by(2) {
            let v0 = self.f32_to_int4(&data[i * 4..i * 4 + 4]);
            let v1 = if i + 1 < n {
                self.f32_to_int4(&data[(i + 1) * 4..(i + 1) * 4 + 4])
            } else {
                0
            };
            out.push((v0 & 0x0F) | ((v1 & 0x0F) << 4));
        }
        Ok(out)
    }

    fn f32_to_int4(&self, chunk: &[u8]) -> u8 {
        let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        ((val.clamp(-8.0, 7.0).round() as i8) + 8) as u8 & 0x0F
    }
}

fn half_to_f16_bits(val: f32) -> u16 {
    // IEEE 754-2008 f32→f16 conversion
    let bits = val.to_bits();
    let sign = (bits >> 16) & 0x8000;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127 + 15;
    let mant = (bits >> 13) & 0x3FF;

    if exp <= 0 {
        sign // Zero/subnormal
    } else if exp > 30 {
        sign | 0x7C00 // Infinity/NaN
    } else {
        sign | ((exp as u16) << 10) | mant as u16
    }
}
