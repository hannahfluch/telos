use crate::error::Result;
use alloc::vec::Vec;
use uefi::{
    boot::{self, ScopedProtocol},
    fs::FileSystem,
    proto::media::fs::SimpleFileSystem,
    CString16,
};

/// Retrieve file data from filesystem of given file name
pub(crate) fn get_file_data(filename: &'static str) -> Result<Vec<u8>> {
    let fs: ScopedProtocol<SimpleFileSystem> = boot::get_image_file_system(boot::image_handle())?;
    let mut fs = FileSystem::new(fs);

    let path = CString16::try_from(filename)?;

    let x = fs.read(path.as_ref())?;
    Ok(x)
}
