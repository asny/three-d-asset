//!
//! Functionality for saving assets. Only available on desktop at the moment.
//!

use super::*;

///
/// Save the assets as files.
///
pub fn save(raw_assets: &RawAssets) -> crate::Result<()> {
    use std::io::prelude::*;
    for (path, bytes) in raw_assets.iter() {
        let mut file = std::fs::File::create(path)?;
        file.write_all(bytes)?;
    }
    Ok(())
}
