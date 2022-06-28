//!
//! Functionality for loading any type of asset runtime on both desktop and web.
//!

use crate::{io::RawAssets, Error, Result};
use std::path::{Path, PathBuf};

///
/// Loads all of the resources in the given paths and returns the [RawAssets] resources.
///
/// Supported functionality:
/// - Loading from disk (relative and absolute paths)
/// - Parsing from data URLs (requires the `data-url` feature flag)
///
/// If downloading resources is also needed, use the [load_async] method instead.
///
#[cfg(not(target_arch = "wasm32"))]
pub fn load(paths: &[impl AsRef<Path>]) -> Result<RawAssets> {
    let mut data_urls = Vec::new();
    let mut local_paths = Vec::new();
    for path in paths.iter() {
        let path = path.as_ref().to_path_buf();
        if is_data_url(&path) {
            data_urls.push(path);
        } else {
            local_paths.push(path);
        }
    }
    let mut raw_assets = RawAssets::new();
    load_from_disk(local_paths, &mut raw_assets)?;
    parse_data_urls(data_urls, &mut raw_assets)?;
    Ok(raw_assets)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_with_dependencies(paths: &[impl AsRef<Path>]) -> Result<RawAssets> {
    let mut raw_assets = load(paths)?;
    let mut dependencies = super::dependencies(&raw_assets);
    while !dependencies.is_empty() {
        let deps = load(&dependencies)?;
        dependencies = super::dependencies(&deps);
        raw_assets.extend(deps);
    }
    Ok(raw_assets)
}

///
/// Async loads all of the resources in the given paths and returns the [RawAssets] resources.
///
/// Supported functionality:
/// - Downloading from URLs relative to the base URL and absolute urls (requires the `http` or `reqwest` feature flag)
/// - Parsing from data URLs (requires the `data-url` feature flag)
///
#[cfg(target_arch = "wasm32")]
pub async fn load_async(paths: &[impl AsRef<Path>]) -> Result<RawAssets> {
    let base_path = base_path();
    let mut urls = Vec::new();
    let mut data_urls = Vec::new();
    for path in paths.iter() {
        let path = path.as_ref().to_path_buf();
        if is_data_url(&path) {
            data_urls.push(path);
        } else if is_absolute_url(&path) {
            urls.push(path);
        } else {
            urls.push(base_path.join(path));
        }
    }
    let mut raw_assets = RawAssets::new();
    load_urls(urls, &mut raw_assets).await?;
    parse_data_urls(data_urls, &mut raw_assets)?;
    Ok(raw_assets)
}

#[allow(rustdoc::bare_urls)]
///
/// Async loads all of the resources in the given paths and returns the [RawAssets] resources.
///
/// Supported functionality:
/// - Downloading from URLs relative to the base URL and absolute urls (requires the `http` or `reqwest` feature flag)
/// - Loading from disk (relative and absolute paths)
/// - Parsing from data URLs (requires the `data-url` feature flag)
///
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_async(paths: &[impl AsRef<Path>]) -> Result<RawAssets> {
    let mut urls = Vec::new();
    let mut data_urls = Vec::new();
    let mut local_paths = Vec::new();
    for path in paths.iter() {
        let path = path.as_ref().to_path_buf();
        if is_data_url(&path) {
            data_urls.push(path);
        } else if is_absolute_url(&path) {
            urls.push(path);
        } else {
            local_paths.push(path);
        }
    }

    let mut raw_assets = RawAssets::new();
    load_urls(urls, &mut raw_assets).await?;
    load_from_disk(local_paths, &mut raw_assets)?;
    parse_data_urls(data_urls, &mut raw_assets)?;
    Ok(raw_assets)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_from_disk(paths: Vec<PathBuf>, raw_assets: &mut RawAssets) -> Result<()> {
    let mut handles = Vec::new();
    for path in paths {
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
        raw_assets.insert(path, bytes);
    }
    Ok(())
}

#[allow(unused_variables)]
async fn load_urls(paths: Vec<PathBuf>, raw_assets: &mut RawAssets) -> Result<()> {
    #[cfg(feature = "reqwest")]
    if paths.len() > 0 {
        let mut handles = Vec::new();
        let client = reqwest::Client::new();
        for path in paths {
            let url = reqwest::Url::parse(path.to_str().unwrap())
                .map_err(|_| Error::FailedParsingUrl(path.to_str().unwrap().to_string()))?;
            handles.push((path, client.get(url).send().await));
        }
        for (path, handle) in handles.drain(..) {
            let bytes = handle
                .map_err(|e| Error::FailedLoadingUrl(path.to_str().unwrap().to_string(), e))?
                .bytes()
                .await
                .map_err(|e| Error::FailedLoadingUrl(path.to_str().unwrap().to_string(), e))?
                .to_vec();
            raw_assets.insert(path, bytes);
        }
    }
    #[cfg(not(feature = "reqwest"))]
    if paths.len() > 0 {
        return Err(Error::FeatureMissing("reqwest".to_string()));
    }
    Ok(())
}

fn parse_data_urls(paths: Vec<PathBuf>, raw_assets: &mut RawAssets) -> Result<()> {
    for path in paths {
        let bytes = parse_data_url(path.to_str().unwrap())?;
        raw_assets.insert(path, bytes);
    }
    Ok(())
}

#[allow(unused_variables)]
fn parse_data_url(path: &str) -> Result<Vec<u8>> {
    #[cfg(feature = "data-url")]
    {
        let url = data_url::DataUrl::process(path)
            .map_err(|e| Error::FailedParsingDataUrl(path.to_string(), format!("{:?}", e)))?;
        let (body, _) = url
            .decode_to_vec()
            .map_err(|e| Error::FailedParsingDataUrl(path.to_string(), format!("{:?}", e)))?;
        Ok(body)
    }
    #[cfg(not(feature = "data-url"))]
    Err(Error::FeatureMissing("data-url".to_string()))
}

fn is_absolute_url(path: &Path) -> bool {
    path.to_str()
        .map(|s| {
            s.find("://").map(|i| i > 0).unwrap_or(false)
                || s.find("//").map(|i| i == 0).unwrap_or(false)
        })
        .unwrap_or(false)
}

fn is_data_url(path: &Path) -> bool {
    path.to_str()
        .map(|s| s.starts_with("data:"))
        .unwrap_or(false)
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

#[cfg(test)]
mod test {

    #[cfg(feature = "data-url")]
    #[test]
    pub fn load_data_url() {
        use super::*;
        use crate::Texture2D;
        let data_url = "data:image/png;base64,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";
        let mut loaded = load(&[data_url, "test_data/data_url.png"]).unwrap();
        let loaded_data_url: Texture2D = loaded.deserialize(data_url).unwrap();
        let loaded_image: Texture2D = loaded.deserialize(".png").unwrap();

        assert_eq!(loaded_data_url, loaded_image);
    }
}
