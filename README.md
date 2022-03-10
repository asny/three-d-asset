# three-d-io

**This is an attempt to do a general input/output crate for 3D assets. Any contribution is very much appreciated!**

A crate for 3D asset input/output which features
- loading any type of asset from disk or from a link into a byte array (on native and web).
- deserializing the loaded byte array into structs representing meshes, textures and materials.
- serializeing the mesh, texture and material structs into a byte array.
- saves any type of asset to disk (on native).

3D format | Deserializing | Serializing
|:------------ | :-------------| :-------------
OBJ | :heavy_check_mark |  :white_large_square
glTF | :heavy_check_mark |  :white_large_square
USDZ | :white_large_square |  :white_large_square
STL | :white_large_square |  :white_large_square

Image format | Deserializing | Serializing
|:------------ | :-------------| :-------------
PNG | :heavy_check_mark |  :heavy_check_mark
JPEG | :heavy_check_mark |  :heavy_check_mark
GIF | :heavy_check_mark |  :heavy_check_mark
WebP | :heavy_check_mark |  :heavy_check_mark
pnm (pbm, pgm, ppm and pam) | :heavy_check_mark |  :heavy_check_mark
TIFF | :heavy_check_mark |  :heavy_check_mark
DDS | :heavy_check_mark |  :heavy_check_mark
BMP | :heavy_check_mark |  :heavy_check_mark
ICO | :heavy_check_mark |  :heavy_check_mark
farbfield | :heavy_check_mark |  :heavy_check_mark
HDR | :heavy_check_mark |  :white_large_square