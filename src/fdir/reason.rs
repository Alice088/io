use thiserror::Error;

#[derive(Debug, Error)]
pub enum Reason {

    #[error("max scheduler tasks")]
    MaxScdulerTasks
}