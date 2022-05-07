//!
//! Functionality for saving resources. Only available on desktop at the moment.
//!

use std::path::Path;

///
/// Save the byte array as a file.
///
pub fn save(path: impl AsRef<Path>, bytes: &[u8]) -> crate::Result<()> {
    let mut file = std::fs::File::create(path)?;
    use std::io::prelude::*;
    file.write_all(bytes)?;
    Ok(())
}
