use std::sync::PoisonError;
use std::sync::mpsc::{RecvError, SendError};

pub enum RegistryError {
    Disconnected,
    LockPoisoned,
}

impl From<PoisonError<()>> for RegistryError {
    fn from(_: PoisonError<()>) -> Self {
        return Self::LockPoisoned
    }
}

impl From<RecvError> for RegistryError {
    fn from(_: RecvError) -> Self {
        return Self::Disconnected
    }
}

impl From<SendError<()>> for RegistryError {
    fn from(_: SendError<()>) -> Self {
        Self::Disconnected
    }
}