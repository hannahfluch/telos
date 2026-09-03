use crate::error::Result;
use alloc::vec::Vec;
use uefi::{
    CString16,
    boot::{self, ScopedProtocol},
    fs::FileSystem,
    proto::media::fs::SimpleFileSystem,
};

/// Read a file from the volume that UEFI used to start this loader.
pub(crate) fn get_file_data(filename: &'static str) -> Result<Vec<u8>> {
    // Tying lookup to the current image avoids accidentally searching another
    // filesystem when several disks are attached.
    let fs: ScopedProtocol<SimpleFileSystem> = boot::get_image_file_system(boot::image_handle())?;
    let mut fs = FileSystem::new(fs);

    // UEFI file paths are null-terminated UTF-16 strings.
    let path = CString16::try_from(filename)?;

    Ok(fs.read(path.as_ref())?)
}
