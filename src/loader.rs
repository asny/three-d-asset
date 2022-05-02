use crate::{Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

///
/// Contains the resources loaded using the [Loader](crate::Loader) and/or manually inserted using the [insert_bytes](Self::insert_bytes) method.
/// Use the [remove_bytes](crate::Loaded::remove_bytes) or [get_bytes](crate::Loaded::get_bytes) function to extract the raw byte array for the loaded resource
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

///
/// Functionality for loading any type of resource runtime on both desktop and web.
///
pub struct Loader {}

impl Loader {
    ///
    /// Loads all of the resources in the given paths then calls `on_done` with all of the [Loaded] resources.
    /// Alternatively use [Loader::load_async] on both web and desktop or [Loader::load_blocking] on desktop.
    ///
    /// **Note:** This method must not be called from an async function. In that case, use [Loader::load_async] instead.
    ///
    pub fn load(paths: &[impl AsRef<Path>], on_done: impl 'static + FnOnce(Result<Loaded>)) {
        #[cfg(target_arch = "wasm32")]
        {
            let paths: Vec<PathBuf> = paths.iter().map(|p| p.as_ref().to_path_buf()).collect();
            wasm_bindgen_futures::spawn_local(async move {
                let loaded = Self::load_async(&paths).await;
                on_done(loaded);
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            on_done(Self::load_blocking(paths));
        }
    }

    ///
    /// Parallel loads all of the resources in the given paths from disk and returns the [Loaded] resources.
    ///
    /// This only loads resources from disk, if downloading resources from URLs is also needed, use the [Loader::load_async] method instead.
    ///
    #[cfg_attr(docsrs, doc(not(target_arch = "wasm32")))]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_blocking(paths: &[impl AsRef<Path>]) -> Result<Loaded> {
        let mut loaded = Loaded::new();
        load_from_disk(
            paths
                .iter()
                .map(|p| p.as_ref().to_path_buf())
                .collect::<Vec<_>>(),
            &mut loaded,
        )?;
        Ok(loaded)
    }

    ///
    /// Async loads all of the resources in the given paths and returns the [Loaded] resources.
    ///
    /// Supports local URLs relative to the base URL ("/my/asset.png") and absolute urls ("https://example.com/my/asset.png").
    ///
    #[cfg(target_arch = "wasm32")]
    pub async fn load_async(paths: &[impl AsRef<Path>]) -> Result<Loaded> {
        let base_path = base_path();
        let mut urls = Vec::new();
        for path in paths.iter() {
            let mut p = path.as_ref().to_path_buf();
            if !is_absolute_url(p.to_str().unwrap()) {
                p = base_path.join(p);
            }
            urls.push(p);
        }
        let mut loaded = Loaded::new();
        load_urls(urls, &mut loaded).await?;
        Ok(loaded)
    }

    #[allow(rustdoc::bare_urls)]
    ///
    /// Loads all of the resources in the given paths and returns the [Loaded] resources.
    /// URLs are downloaded async and resources on disk are loaded in parallel.
    ///
    /// Supports local URLs relative to the base URL ("/my/asset.png") and absolute urls ("https://example.com/my/asset.png").
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn load_async(paths: &[impl AsRef<Path>]) -> Result<Loaded> {
        let mut urls = Vec::new();
        let mut local_paths = Vec::new();
        for path in paths.iter() {
            let path = path.as_ref().to_path_buf();
            if is_absolute_url(path.to_str().unwrap()) {
                urls.push(path);
            } else {
                local_paths.push(path);
            }
        }

        let mut loaded = Loaded::new();
        load_urls(urls, &mut loaded).await?;
        load_from_disk(local_paths, &mut loaded)?;
        Ok(loaded)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_from_disk(mut paths: Vec<PathBuf>, loaded: &mut Loaded) -> Result<()> {
    let mut handles = Vec::new();
    for path in paths.drain(..) {
        handles.push((
            path.clone(),
            std::thread::spawn(move || std::fs::read(path)),
        ));
    }

    for (path, handle) in handles.drain(..) {
        let bytes = handle
            .join()
            .unwrap()
            .map_err(|e| Error::FailedLoading(path.to_str().unwrap().to_string(), e))?;
        loaded.loaded.insert(path, bytes);
    }
    Ok(())
}

#[cfg(feature = "reqwest")]
async fn load_urls(mut paths: Vec<PathBuf>, loaded: &mut Loaded) -> Result<()> {
    if paths.len() > 0 {
        let mut handles = Vec::new();
        let client = reqwest::Client::new();
        for path in paths.drain(..) {
            let url = reqwest::Url::parse(path.to_str().unwrap())?;
            handles.push((path, client.get(url).send().await));
        }
        for (path, handle) in handles.drain(..) {
            let bytes = handle
                .map_err(|e| Error::FailedLoadingUrl(path.to_str().unwrap().to_string(), e))?
                .bytes()
                .await
                .map_err(|e| Error::FailedLoadingUrl(path.to_str().unwrap().to_string(), e))?
                .to_vec();
            loaded.loaded.insert(path, bytes);
        }
    }
    Ok(())
}

#[cfg(not(feature = "reqwest"))]
async fn load_urls(paths: Vec<PathBuf>, _loaded: &mut Loaded) -> Result<()> {
    if paths.is_empty() {
        Ok(())
    } else {
        let url = paths[0].to_str().unwrap().to_owned();
        Err(Error::FailedLoadingUrl(url))
    }
}

fn is_absolute_url(path: &str) -> bool {
    path.find("://").map(|i| i > 0).unwrap_or(false)
        || path.find("//").map(|i| i == 0).unwrap_or(false)
}

#[cfg(target_arch = "wasm32")]
fn base_path() -> PathBuf {
    let base_url = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .url()
        .unwrap();
    if !base_url.ends_with('/') {
        PathBuf::from(base_url).parent().unwrap().to_path_buf()
    } else {
        PathBuf::from(base_url)
    }
}
