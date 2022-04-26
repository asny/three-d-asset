#[cfg(all(feature = "obj", feature = "image"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "obj", feature = "image"))))]
mod obj;
#[doc(inline)]
#[cfg(all(feature = "obj", feature = "image"))]
pub use obj::*;

#[cfg(all(feature = "gltf", feature = "image"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "gltf", feature = "image"))))]
mod gltf;
#[doc(inline)]
#[cfg(all(feature = "gltf", feature = "image"))]
pub use self::gltf::*;

#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
mod img;
#[cfg(feature = "image")]
#[doc(inline)]
pub use img::*;

mod vol;
#[doc(inline)]
pub use vol::*;
