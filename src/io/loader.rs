//!
//! Functionality for loading any type of asset runtime on both desktop and web.
//!

use crate::{io::RawAssets, Error, Result};
use std::path::{Path, PathBuf};

///
/// Parallel loads all of the resources in the given paths from disk and returns the [RawAssets] resources.
///
/// This only loads resources from disk, if downloading resources from URLs is also needed, use the [load_async] method instead.
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

///
/// Async loads all of the resources in the given paths and returns the [RawAssets] resources.
///
/// Supports local URLs relative to the base URL ("/my/asset.png") and, if the `http` feature is enabled, absolute urls ("https://example.com/my/asset.png").
///
#[cfg(all(feature = "reqwest", target_arch = "wasm32"))]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
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
/// Loads all of the resources in the given paths and returns the [RawAssets] resources.
/// URLs are downloaded async (requires the `http` feature) and resources on disk are loaded in parallel.
///
/// Supports local URLs relative to the base URL ("/my/asset.png") and absolute urls ("https://example.com/my/asset.png").
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
fn load_from_disk(mut paths: Vec<PathBuf>, raw_assets: &mut RawAssets) -> Result<()> {
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
        raw_assets.insert(path, bytes);
    }
    Ok(())
}

#[cfg(feature = "reqwest")]
async fn load_urls(mut paths: Vec<PathBuf>, raw_assets: &mut RawAssets) -> Result<()> {
    if paths.len() > 0 {
        let mut handles = Vec::new();
        let client = reqwest::Client::new();
        for path in paths.drain(..) {
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
    Ok(())
}

#[cfg(not(feature = "reqwest"))]
async fn load_urls(paths: Vec<PathBuf>, _raw_assets: &mut RawAssets) -> Result<()> {
    if paths.is_empty() {
        Ok(())
    } else {
        let url = paths[0].to_str().unwrap().to_owned();
        Err(Error::FailedLoadingUrl(url))
    }
}

fn parse_data_urls(mut paths: Vec<PathBuf>, raw_assets: &mut RawAssets) -> Result<()> {
    for path in paths.drain(..) {
        let bytes = crate::io::parse_data_url(path.to_str().unwrap())?;
        raw_assets.insert(path, bytes);
    }
    Ok(())
}

pub(crate) fn parse_data_url(path: &str) -> Result<Vec<u8>> {
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
    Err(Error::FeatureMissing(
        "data-url".to_string(),
        path.to_string(),
    ))
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
