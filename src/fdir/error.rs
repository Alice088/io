use crate::fdir::reason::Reason;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("software error by: {0}")]
    Software(Reason),

    #[error("hardware error by: {0}")]
    Hardware(Reason)
}