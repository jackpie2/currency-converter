use std::{collections::HashMap, env};

use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{converter::Converter, helpers::ConverterError};

use super::{ConverterDataSource, CurrencyList};

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    last_updated_at: String,
    base: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Currency {
    code: String,
    value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrencyApi {
    pub meta: Meta,
    pub data: HashMap<String, Currency>,
}

impl CurrencyApi {
    async fn fetch(base: &str, target: &str) -> Result<Self, ConverterError> {
        let url = format!(
            "https://api.currencyapi.com/v3/latest?apikey={}&currencies={}&base_currency={}",
            env::var("CURRENCY_API_KEY").expect("CURRENCY_API_KEY is not set"),
            target,
            base
        );
        let request = reqwest::get(&url).await;
        let response: Response = match request {
            Ok(val) => {
                match val.status() {
                    reqwest::StatusCode::OK => val,
                    status_code => match status_code {
                        reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
                            return Err(ConverterError::FetchError(format!(
                                    "The request was invalid. Please check that your inputs were correct and are supported: {} -> {}",
                                    base, target
                                )));
                        }
                        reqwest::StatusCode::TOO_MANY_REQUESTS => {
                            return Err(ConverterError::FetchError("You have reached the rate limit for the API. Please try again later.".to_string()));
                        }
                        reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                            return Err(ConverterError::FetchError("There was an error on the API server side. Please try again later.".to_string()));
                        }
                        reqwest::StatusCode::UNAUTHORIZED => {
                            return Err(ConverterError::FetchError("The API key is invalid. Please check that the CURRENCY_API_KEY environment variable is set correctly.".to_string()));
                        }
                        _ => {
                            return Err(ConverterError::FetchError(format!(
                                "There was a network error while fetching data from the API, status code: {}",
                                status_code
                            )));
                        }
                    },
                }
            }
            Err(err) => {
                return Err(ConverterError::FetchError(format!(
                    "There was a network error while fetching data from the API: {}",
                    err
                )));
            }
        };

        let mut currency_data: CurrencyApi = match response.json().await {
            Ok(val) => val,
            Err(error) => {
                return Err(ConverterError::ParseError(error.to_string()));
            }
        };

        currency_data.meta.base = Some(base.to_string());

        Ok(currency_data)
    }

    fn into_converter(self) -> Converter {
        let base = self
            .meta
            .base
            .expect("The base currency was not set in the API response.");

        let target = self
            .data
            .keys()
            .next()
            .expect("The target currency was not set in the API response.")
            .to_string();

        let rate = self
            .data
            .values()
            .next()
            .expect("The rate for the target currency was not set in the API response.")
            .value;

        Converter::new(base, target, rate)
    }
}

impl ConverterDataSource for CurrencyApi {
    async fn load(base: &str, target: &str) -> Result<Converter, crate::helpers::ConverterError> {
        let api_data = CurrencyApi::fetch(base, target).await?;
        Ok(api_data.into_converter())
    }

    async fn list() -> Result<CurrencyList, ConverterError> {
        let url = format!(
            "https://api.currencyapi.com/v3/currencies?apikey={}",
            env::var("CURRENCY_API_KEY").expect("CURRENCY_API_KEY is not set")
        );
        let request = reqwest::get(&url).await;
        let response: Response = match request {
            Ok(val) => {
                match val.status() {
                    reqwest::StatusCode::OK => val,
                    status_code => match status_code {
                        reqwest::StatusCode::TOO_MANY_REQUESTS => {
                            return Err(ConverterError::FetchError("You have reached the rate limit for the API. Please try again later.".to_string()));
                        }
                        reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                            return Err(ConverterError::FetchError("There was an error on the API server side. Please try again later.".to_string()));
                        }
                        reqwest::StatusCode::UNAUTHORIZED => {
                            return Err(ConverterError::FetchError("The API key is invalid. Please check that the CURRENCY_API_KEY environment variable is set correctly.".to_string()));
                        }
                        _ => {
                            return Err(ConverterError::FetchError(format!(
                                "There was a network error while fetching data from the API, status code: {}",
                                status_code
                            )));
                        }
                    },
                }
            }
            Err(err) => {
                return Err(ConverterError::FetchError(format!(
                    "There was a network error while fetching data from the API: {}",
                    err
                )));
            }
        };
        let json = response.json::<serde_json::Value>().await;

        let list = match json {
            Ok(val) => match val["data"].as_object() {
                Some(data) => data.keys().map(|x| x.to_string()).collect(),
                None => {
                    return Err(ConverterError::ParseError(
                        "The API response was not in the expected format.".to_string(),
                    ));
                }
            },
            Err(err) => {
                return Err(ConverterError::ParseError(err.to_string()));
            }
        };

        let currency_list = CurrencyList { currencies: list };
        Ok(currency_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_converter() {
        let api_data = CurrencyApi {
            meta: Meta {
                last_updated_at: "2021-01-01".to_string(),
                base: Some("USD".to_string()),
            },
            data: {
                let mut map = HashMap::new();
                map.insert(
                    "EUR".to_string(),
                    Currency {
                        code: "EUR".to_string(),
                        value: 0.85,
                    },
                );
                map
            },
        };

        let converter = api_data.into_converter();
        assert_eq!(converter.base, "USD");
        assert_eq!(converter.target, "EUR");
        assert_eq!(converter.rate, 0.85);
    }
}
