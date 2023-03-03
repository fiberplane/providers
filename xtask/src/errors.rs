use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Unknown command")]
    UnknownCommand,
}
