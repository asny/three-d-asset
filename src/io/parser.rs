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

#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
mod img;
#[cfg(feature = "image")]
#[doc(inline)]
pub use img::*;

#[cfg(feature = "vol")]
#[cfg_attr(docsrs, doc(cfg(feature = "vol")))]
mod vol;
#[cfg(feature = "vol")]
#[doc(inline)]
pub use vol::*;
