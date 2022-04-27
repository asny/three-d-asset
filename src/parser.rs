#[cfg(feature = "obj")]
#[cfg_attr(docsrs, doc(cfg(feature = "obj")))]
mod obj;
#[doc(inline)]
#[cfg(feature = "obj")]
pub use obj::*;

#[cfg(feature = "gltf")]
#[cfg_attr(docsrs, doc(cfg(feature = "gltf")))]
mod gltf;
#[doc(inline)]
#[cfg(feature = "gltf")]
pub use self::gltf::*;

#[cfg(any(feature = "png", feature = "jpeg"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "png", feature = "jpeg"))))]
mod img;
#[cfg(any(feature = "png", feature = "jpeg"))]
#[doc(inline)]
pub use img::*;

#[cfg(feature = "vol")]
#[cfg_attr(docsrs, doc(cfg(feature = "vol")))]
mod vol;
#[cfg(feature = "vol")]
#[doc(inline)]
pub use vol::*;
