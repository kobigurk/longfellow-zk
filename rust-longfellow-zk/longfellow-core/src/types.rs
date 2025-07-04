use serde::{Deserialize, Serialize};

pub type Field = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polynomial {
    pub coefficients: Vec<Field>,
}

impl Polynomial {
    pub fn new(coefficients: Vec<Field>) -> Self {
        Self { coefficients }
    }

    pub fn zero() -> Self {
        Self {
            coefficients: vec![],
        }
    }

    pub fn degree(&self) -> usize {
        if self.coefficients.is_empty() {
            0
        } else {
            self.coefficients.len() - 1
        }
    }
}