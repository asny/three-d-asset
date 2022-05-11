//!
//! Contains functionality to load any type of asset runtime as well as parsers for common 3D assets.
//! Also includes functionality to save data which is limited to native.
//!

mod loader;
#[doc(inline)]
pub use loader::*;

mod raw_assets;
#[doc(inline)]
pub use raw_assets::*;

mod parser;
#[doc(inline)]
pub use parser::*;

#[cfg(not(target_arch = "wasm32"))]
mod saver;
#[doc(inline)]
#[cfg(not(target_arch = "wasm32"))]
pub use saver::*;

pub trait Deserialize: Sized {
    fn deserialize(
        path: impl AsRef<std::path::Path>,
        raw_assets: &mut RawAssets,
    ) -> crate::Result<Self>;
}

pub trait Serialize: Sized {
    fn serialize(&self, path: impl AsRef<std::path::Path>) -> crate::Result<RawAssets>;
}
