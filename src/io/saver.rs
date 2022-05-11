//!
//! Functionality for saving assets. Only available on desktop at the moment.
//!

use super::*;
use std::path::Path;

///
/// Save the asset as a file.
///
pub fn save(path: impl AsRef<Path>, asset: impl Serialize) -> crate::Result<()> {
    use std::io::prelude::*;
    let raw_assets = asset.serialize(path)?;
    for (path, bytes) in raw_assets.iter() {
        let mut file = std::fs::File::create(path)?;
        file.write_all(&bytes)?;
    }
    Ok(())
}
