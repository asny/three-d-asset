use crate::{io::Deserialize, Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

///
/// Contains the resources loaded using one of the loader functions and/or manually inserted using the [Loaded::insert_bytes] method.
/// Use the [Loaded::remove_bytes] or [Loaded::get_bytes] function to extract the raw byte array for the loaded resource
/// or one of the other methods to both extract and deserialize a loaded resource.
///
#[derive(Default)]
pub struct Loaded {
    loaded: HashMap<PathBuf, Vec<u8>>,
}

impl Loaded {
    ///
    /// Constructs a new empty set of loaded files. Use this together with [insert_bytes](Self::insert_bytes) to load resources
    /// from an unsuported source and then parse them as usual using the functionality on Loaded.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    ///
    /// Remove and returns the loaded byte array for the resource at the given path.
    /// The byte array then has to be deserialized to whatever type this resource is (image, 3D model etc.).
    ///
    pub fn remove_bytes(&mut self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        if let Some((_, bytes)) = self.loaded.remove_entry(path.as_ref()) {
            Ok(bytes)
        } else {
            let mut p = path.as_ref().to_str().unwrap().to_owned();
            if p.ends_with(".jpeg") {
                p = p[0..p.len() - 2].to_string();
            } else if p.ends_with(".jpg") {
                p = p[0..p.len() - 1].to_string();
            }
            let key = self
                .loaded
                .iter()
                .find(|(k, _)| k.to_str().unwrap().contains(&p))
                .ok_or(Error::NotLoaded(p))?
                .0
                .clone();
            Ok(self.loaded.remove(&key).unwrap())
        }
    }

    ///
    /// Returns a reference to the loaded byte array for the resource at the given path.
    /// The byte array then has to be deserialized to whatever type this resource is (image, 3D model etc.).
    ///
    pub fn get_bytes(&self, path: impl AsRef<Path>) -> Result<&[u8]> {
        if let Some(bytes) = self.loaded.get(path.as_ref()) {
            Ok(bytes.as_ref())
        } else {
            let mut p = path.as_ref().to_str().unwrap().to_owned();
            if p.ends_with(".jpeg") {
                p = p[0..p.len() - 2].to_string();
            } else if p.ends_with(".jpg") {
                p = p[0..p.len() - 1].to_string();
            }
            let key = self
                .loaded
                .iter()
                .find(|(k, _)| k.to_str().unwrap().contains(&p))
                .ok_or(Error::NotLoaded(p))?
                .0;
            Ok(self.loaded.get(key).unwrap())
        }
    }

    ///
    /// Inserts the given bytes into the set of loaded files which is useful if you want to load the data from an unsuported source.
    /// The files can then be parsed as usual using the functionality on Loaded.
    ///
    pub fn insert_bytes(&mut self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        self.loaded.insert(path.as_ref().to_path_buf(), bytes);
    }

    pub fn deserialize<T: Deserialize>(&mut self, path: impl AsRef<Path>) -> Result<T> {
        T::deserialize(self, path)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, PathBuf, Vec<u8>> {
        self.loaded.iter()
    }
}

impl std::fmt::Debug for Loaded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Loaded");
        for (key, value) in self.loaded.iter() {
            d.field("path", key);
            d.field("byte length", &value.len());
        }
        d.finish()
    }
}
