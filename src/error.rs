use std::{error::Error, fmt::Display};

pub const UTF8_PARSER_MSG: &str = "UTF-8 parser error";


#[derive(Debug)]
pub struct MadxLsError {
    pub message: String
}

impl MadxLsError {
    pub fn new<T, S: Into<String>>(message: S) -> Result<T, Self> {
        Err(Self {
            message: message.into()
        })
    }
}

impl Display for MadxLsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MadxLsError{}

