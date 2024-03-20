use std::{fmt::Display, process::exit};

pub fn error_and_exit<T: Display>(error: T) -> ! {
    eprintln!("{}", error);
    exit(1);
}

#[derive(Debug)]
pub enum ConverterError {
    FetchError(String),
    ParseError(String),
}

impl Display for ConverterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConverterError::FetchError(error) => {
                write!(f, "Error while fetching currency data from API: {}", error)
            }
            ConverterError::ParseError(error) => {
                write!(f, "Error while parsing currency data from API: {}", error)
            }
        }
    }
}
