# three-d-asset

**This is an attempt to do a general crate for loading, saving and editing 3D assets. The idea is that it should be possible to use it as a base for any type of visualization or advanced editing tools, a bit like the `image` crate, just for 3D assets. Contributions are very much appreciated!**

The crate contain a set of common assets that are useful when doing graphics which can be loaded using the `io` module or constructed manually.
When in memory, the assets can be for example be
- visualised, for example using the [three-d](https://github.com/asny/three-d) crate or in a CPU ray tracer
- imported into a rust-based game engine
- edited and saved again

### 3D model

| Format | Deserializing | Serializing | Feature | 
| ------------ | -------------| ------------- | ------------- |
| OBJ | :heavy_check_mark: |  :white_large_square: | `obj` |
| GLTF | :heavy_check_mark: |  :white_large_square: | `gltf` |

### Texture

| Format | Deserializing | Serializing | Feature | 
| ------------ | ------------- | ------------- | ------------- |
| PNG | :heavy_check_mark: |  :heavy_check_mark: | `png` |
| JPEG | :heavy_check_mark: |  :heavy_check_mark: | `jpeg` |
| HDR | :heavy_check_mark: |  :white_large_square: | `hdr` |
| GIF | :heavy_check_mark: |  :heavy_check_mark: | `gif` |
| TGA | :heavy_check_mark: |  :heavy_check_mark: | `tga` |
| TIFF | :heavy_check_mark: |  :heavy_check_mark: | `tiff` |
| BMP | :heavy_check_mark: |  :heavy_check_mark: | `bmp` |

### Volume

| Format | Deserializing | Serializing | Feature | 
| ------------ | ------------- | ------------- | ------------- |
| VOL | :heavy_check_mark: |  :white_large_square: | `vol` |