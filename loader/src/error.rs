pub(crate) type Result<T> = core::result::Result<T, LoaderError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum LoaderError {
    #[error("Uefi error: {0}")]
    Uefi(#[from] uefi::Error),
    #[error("Fs error: {0}")]
    UefiFs(#[from] uefi::fs::Error),
    #[error("FromStr error: {0}")]
    UefiFromStr(#[from] uefi::data_types::FromStrError),
    #[error("Invalid filename: {0}")]
    InvalidFile(&'static str),
    #[error("Psf error: {0}")]
    Psf(#[from] PsfParseError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PsfParseError {
    #[error("Insufficient font data for PSF header")]
    InsufficientDataForPSFHeader,
    #[error("Insufficient font data for PSF1 data")]
    InsufficientDataForPSF1,
    #[error("Insufficient font data for PSF2 data")]
    InsufficientDataForPSF2,
    #[error("Unrecognized PSF header magic: {0}")]
    InvalidPSFMagic(u32),
}
