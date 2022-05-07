//!
//! Contains functionality to load any type of asset runtime as well as parsers for common 3D assets.
//! Also includes functionality to save data which is limited to native.
//!

mod loader;
#[doc(inline)]
pub use loader::*;

mod loaded;
#[doc(inline)]
pub use loaded::*;

mod parser;
#[doc(inline)]
pub use parser::*;

#[cfg(not(target_arch = "wasm32"))]
mod saver;
#[doc(inline)]
#[cfg(not(target_arch = "wasm32"))]
pub use saver::*;
