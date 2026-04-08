# `three-d-asset`

[![crates.io](https://img.shields.io/crates/v/three-d-asset.svg)](https://crates.io/crates/three-d-asset)
[![Docs.rs](https://docs.rs/three-d-asset/badge.svg)](https://docs.rs/three-d-asset)
[![Continuous integration](https://github.com/asny/three-d-asset/actions/workflows/rust.yml/badge.svg)](https://github.com/asny/three-d-asset/actions/workflows/rust.yml)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/asny/three-d-asset/blob/main/LICENSE)

**This is an attempt to do a general crate for loading, saving and editing 3D assets. The idea is that it should be possible to use it as a base for any type of visualization or advanced editing tools, a bit like the `image` crate, just for 3D assets. Contributions are very much appreciated!**

The crate contain a set of common assets that are useful when doing graphics which can be loaded using the `io` module or constructed manually.
When in memory, the assets can be for example be

- visualised, for example using the [three-d](https://github.com/asny/three-d) crate or in a CPU ray tracer
- imported into a rust-based game engine
- edited and saved again

### Model

| Format   | Deserialize | Serialize | Feature |
|----------|-------------|-----------|---------|
| OBJ/MTL  | ✅          | ❌        | `obj`   |
| GLTF/GLB | ✅          | ❌        | `gltf`  |
| STL      | ✅          | ❌        | `stl`   |
| 3MF      | ✅          | ✅        | `3mf`   |

### Texture2D

| Format | Deserialize | Serialize | Feature |
|--------|-------------|-----------|---------|
| PNG    | ✅          | ✅        | `png`   |
| JPEG   | ✅          | ✅        | `jpeg`  |
| HDR    | ✅          | ❌        | `hdr`   |
| GIF    | ✅          | ✅        | `gif`   |
| TGA    | ✅          | ✅        | `tga`   |
| TIFF   | ✅          | ✅        | `tiff`  |
| BMP    | ✅          | ✅        | `bmp`   |
| SVG    | ✅          | ❌        | `svg`   |

### PointCloud

| Format | Deserialize | Serialize | Feature |
| ------ | ----------- | --------- | ------- |
| PCD    | ✅          | ❌        | `pcd`   |

### VoxelGrid

| Format | Deserialize | Serialize | Feature |
| ------ | ----------- | --------- | ------- |
| VOL    | ✅          | ❌        | `vol`   |
