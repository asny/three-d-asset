use crate::{io::Deserialize, Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

///
/// Contains raw assets using one of the loader functions and/or manually inserted using the [RawAssets::insert] or [RawAssets::extend] methods.
/// Use the [RawAssets::remove] or [RawAssets::get] function to extract the raw byte array for the assets
/// or [RawAssets::deserialize] to deserialize an asset.
///
#[derive(Default)]
pub struct RawAssets(HashMap<PathBuf, Vec<u8>>);

impl RawAssets {
    ///
    /// Constructs a new empty set of raw assets.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    ///
    /// Remove and returns the raw byte array for the resource at the given path.
    ///
    pub fn remove(&mut self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        Ok(self.0.remove(&self.match_path(path)?).unwrap())
    }

    ///
    /// Returns a reference to the raw byte array for the resource at the given path.
    ///
    pub fn get(&self, path: impl AsRef<Path>) -> Result<&[u8]> {
        Ok(self.0.get(&self.match_path(path)?).unwrap())
    }

    pub(crate) fn match_path(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = path.as_ref();
        if self.0.contains_key(path) {
            Ok(path.to_path_buf())
        } else {
            let mut p = path.to_str().unwrap().to_owned();
            if p.ends_with(".jpeg") {
                p = p[0..p.len() - 2].to_string();
            } else if p.ends_with(".jpg") {
                p = p[0..p.len() - 1].to_string();
            }
            self.0
                .iter()
                .find(|(k, _)| k.to_str().unwrap().contains(&p))
                .map(|(k, _)| k.clone())
                .ok_or(Error::NotLoaded(p))
        }
    }

    ///
    /// Inserts the given bytes into the set of raw assets which is useful if you want to add data from an unsuported source
    /// to be able to use either the [RawAssets::deserialize] functionality or [crate::io::save] functionality.
    ///
    pub fn insert(&mut self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        self.0.insert(path.as_ref().to_path_buf(), bytes);
    }

    pub fn extend(&mut self, mut raw_assets: Self) -> &mut Self {
        for (k, v) in raw_assets.0.drain() {
            self.0.insert(k, v);
        }
        self
    }

    pub fn deserialize<T: Deserialize>(&mut self, path: impl AsRef<Path>) -> Result<T> {
        T::deserialize(self, path)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, PathBuf, Vec<u8>> {
        self.0.iter()
    }
}

impl std::fmt::Debug for RawAssets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("RawAssets");
        for (key, value) in self.0.iter() {
            d.field("path", key);
            d.field("byte length", &value.len());
        }
        d.finish()
    }
}
