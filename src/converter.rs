use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Converter {
    pub base: String,
    pub target: String,
    pub rate: f64,
}

impl Converter {
    pub fn new(base: String, target: String, rate: f64) -> Self {
        Converter { base, target, rate }
    }

    pub fn convert(&self, amount: f64) -> f64 {
        amount * self.rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert() {
        let converter = Converter::new("USD".to_string(), "EUR".to_string(), 0.85);
        assert_eq!(converter.convert(100.0), 85.0);
    }
}
