use thiserror::Error;

#[derive(Error, Debug)]
pub enum RivuletError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Feed parsing error: {0}")]
    FeedParse(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Feed not found: {0}")]
    FeedNotFound(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RivuletError>;
