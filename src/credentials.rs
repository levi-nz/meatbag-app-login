use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use crate::credentials::ParseError::IoError;

/// A username + password combination.
#[derive(Clone)]
pub(crate) struct Account {
    /// The account's username.
    pub(crate) username: String,

    /// The account's password.
    pub(crate) password: String
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.username, self.password)
    }
}

/// A parsing error.
#[derive(Debug)]
pub(crate) enum ParseError {
    /// An IO error.
    IoError {
        error: std::io::Error
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError { error } => write!(f, "IO error: {}", error)
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        IoError {
            error
        }
    }
}

/// Parses credentials from the given file path.
pub(crate) fn parse_credentials(file_path: &str) -> Result<VecDeque<Account>, ParseError> {
    Ok(
        std::fs::read_to_string(file_path)?
            .lines()
            .map(|entry| entry.split(":"))
            .filter_map(|mut parts| if let (Some(username), Some(password)) = (parts.next(), parts.next()) {
                Some(Account {
                    username: username.to_string(),
                    password: password.to_string()
                })
            } else {
                None
            })
            .collect()
    )
}
