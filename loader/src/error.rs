pub(crate) type Result<T> = core::result::Result<T, LoaderError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum LoaderError {
    #[error("Uefi error: {0}")]
    Uefi(#[from] uefi::Error),
    #[error("Fs error: {0}")]
    UefiFs(#[from] uefi::fs::Error),
    #[error("FromStr error: {0}")]
    UefiFromStr(#[from] uefi::data_types::FromStrError),
    #[error("Psf error: {0}")]
    Psf(#[from] PsfParseError),
    #[error("ELF parsing error (goblin): {0}")]
    Goblin(#[from] goblin::error::Error),
    #[error("Kernel loading error: {0}")]
    KernelLoad(#[from] KernelLoadError),
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

#[derive(Debug, thiserror::Error)]
pub(crate) enum KernelLoadError {
    #[error("Kernel must be a little-endian x86-64 executable ELF")]
    InvalidFormat,
    #[error("Kernel ELF contains no loadable segments")]
    NoLoadableSegments,
    #[error("Kernel ELF contains an invalid load segment")]
    InvalidSegment,
    #[error("Kernel entry point is not in an executable load segment")]
    InvalidEntryPoint,
}
