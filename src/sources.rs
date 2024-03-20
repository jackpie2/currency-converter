use std::fmt::Display;

use crate::converter::Converter;

pub mod currency_api;

pub trait ConverterDataSource {
    async fn load(base: &str, target: &str) -> Result<Converter, crate::helpers::ConverterError>;
    async fn list() -> Result<CurrencyList, crate::helpers::ConverterError>;
}

pub struct CurrencyList {
    pub currencies: Vec<String>,
}

impl Display for CurrencyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Available currencies: {}", self.currencies.join(", "))
    }
}
