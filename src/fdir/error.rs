use crate::fdir::reason::Reason;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("software error by: {0}")]
    Software(Reason)
}