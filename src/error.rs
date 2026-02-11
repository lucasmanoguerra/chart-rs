use thiserror::Error;

pub type ChartResult<T> = Result<T, ChartError>;

#[derive(Debug, Error)]
pub enum ChartError {
    #[error("invalid viewport size: width={width}, height={height}")]
    InvalidViewport { width: u32, height: u32 },

    #[error("invalid data: {0}")]
    InvalidData(String),
}
