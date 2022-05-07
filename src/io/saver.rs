//!
//! Functionality for saving assets. Only available on desktop at the moment.
//!

use super::*;
use std::path::Path;

///
/// Save the asset as a file.
///
pub fn save(path: impl AsRef<Path>, asset: impl Asset) -> crate::Result<()> {
    let mut file = std::fs::File::create(path)?;
    use std::io::prelude::*;
    file.write_all(&asset.to_bytes()?)?;
    Ok(())
}
