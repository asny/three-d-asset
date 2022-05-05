//!
//! Contains functionality to load any type of asset runtime as well as parsers for different image and 3D model formats.
//! The parsers will output into data types defined in the [three-d-data-types](https://github.com/asny/three-d-data-types) crate.
//! Also includes functionality to save data which is limited to native.
//!

mod loader;
#[doc(inline)]
pub use loader::*;

mod parser;
#[doc(inline)]
pub use parser::*;

#[cfg(not(target_arch = "wasm32"))]
mod saver;
#[doc(inline)]
#[cfg(not(target_arch = "wasm32"))]
pub use saver::*;
