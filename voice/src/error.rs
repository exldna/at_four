#[derive(thiserror::Error, Debug)]
pub enum VoiceLibError {
    /// Represents error caused by platform API.
    #[error(transparent)]
    PlatformSpecific(#[from] windows::core::Error),

    /// Represents error occurred by string building.
    #[error(transparent)]
    StringifyError(#[from] std::string::FromUtf16Error),

    /// Represents unknown error. ¯\_(ツ)_/¯
    #[error("unknown voice lib error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, VoiceLibError>;
