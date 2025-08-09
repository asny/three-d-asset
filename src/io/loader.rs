//!
//! Functionality for loading any type of asset runtime on both desktop and web.
//!

use crate::{io::RawAssets, Error, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// User Agent string for three-d-asset
#[cfg(feature = "reqwest")]
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION"));

///
/// Run a future to completion, returning any [`Output`].
///
/// NOTE: This creates a tokio runtime to run the future in, so this should
/// likely be called on some top-level future and not in a loop.
///
/// [`Output`]: std::future::Future::Output
///
#[cfg(not(target_arch = "wasm32"))]
fn block_on<F>(f: F) -> F::Output
where
    F: std::future::Future,
{
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

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
    block_on(load_async(paths))
}

///
/// Async loads all of the resources in the given paths and returns the [RawAssets] resources.
///
/// Supported functionality:
/// - Downloading from URLs relative to the base URL and absolute urls (requires the `http` or `reqwest` feature flag)
/// - Parsing from data URLs (requires the `data-url` feature flag)
/// - *** Native only *** Loading from disk (relative and absolute paths)
///
pub async fn load_async(paths: &[impl AsRef<Path>]) -> Result<RawAssets> {
    let mut raw_assets = load_async_single(paths).await?;
    let mut dependencies = super::get_dependencies(&raw_assets);
    while !dependencies.is_empty() {
        let deps = load_async_single(&dependencies).await?;
        dependencies = super::get_dependencies(&deps);
        raw_assets.extend(deps);
    }
    Ok(raw_assets)
}

///
/// Load paths, but not any of their dependencies (eg. loading an obj will not
/// load it's textures in turn)
///
#[cfg(target_arch = "wasm32")]
async fn load_async_single(paths: &[impl AsRef<Path>]) -> Result<RawAssets> {
    let base_path = base_path();
    let mut urls = HashSet::new();
    let mut data_urls = HashSet::new();
    for path in paths.iter() {
        let path = path.as_ref().to_path_buf();
        if is_data_url(&path) {
            data_urls.insert(path);
        } else if is_absolute_url(&path) {
            urls.insert(path);
        } else {
            urls.insert(base_path.join(path));
        }
    }
    let mut raw_assets = load_urls(urls).await?;
    parse_data_urls(data_urls, &mut raw_assets)?;
    Ok(raw_assets)
}

///
/// Load paths, but not any of their dependencies (eg. loading an obj will not
/// load it's textures in turn)
///
#[cfg(not(target_arch = "wasm32"))]
async fn load_async_single(paths: &[impl AsRef<Path>]) -> Result<RawAssets> {
    let mut urls = HashSet::new();
    let mut data_urls = HashSet::new();
    let mut local_paths = HashSet::new();
    for path in paths.iter() {
        let path = path.as_ref().to_path_buf();
        if is_data_url(&path) {
            data_urls.insert(path);
        } else if is_absolute_url(&path) {
            urls.insert(path);
        } else {
            local_paths.insert(path);
        }
    }

    let mut raw_assets = RawAssets::new();
    // load from network and disk in parallel, returning on the first error
    match tokio::try_join!(load_urls(urls), load_from_disk(local_paths)) {
        Ok((urls_assets, disk_assets)) => {
            raw_assets.extend(urls_assets);
            raw_assets.extend(disk_assets);
        }
        Err(e) => return Err(e),
    }
    // This function is cpu bound and does not need to be async fn, however it's
    // non-trivial if the n of data_urls is large, it may make sense to process
    // them in parallel in the future.
    parse_data_urls(data_urls, &mut raw_assets)?;
    Ok(raw_assets)
}

/// Load assets from disk.
#[cfg(not(target_arch = "wasm32"))]
async fn load_from_disk<Ps>(paths: Ps) -> Result<RawAssets>
where
    Ps: IntoIterator<Item = PathBuf>,
{
    let mut raw_assets = RawAssets::new();
    let mut tasks = tokio::task::JoinSet::new();

    for path in paths {
        // Note: This will spawn all of the tasks at once (which are cheap, only
        // 64kb per task), but Tokio will very likely schedule them to run in
        // sequence in a dedicated thread. This is a good thing since loading
        // many files from *disk* at the same time will likely hurt performance
        // due to memory locality issues, especially with spinning disks.
        // Letting the runtime decide what to do is probably best here as in
        // the future it might use underlying native async io features of the OS
        // rather than an IO thread/pool.
        tasks.spawn(async move {
            let bytes = tokio::fs::read(&path)
                .await
                .map_err(|e| Error::FailedLoading(path.to_string_lossy().into(), e))?;

            Ok((path, bytes))
        });
    }

    // Iterate over the `res`ults of the tasks as they complete
    while let Some(Ok(res)) = tasks.join_next().await {
        // We don't care about Some(Err(e)) as this only happens if the join
        // fails which can only happen if a task doesn't complete but that can't
        // happpen because the task code in the above for loop can't panic.
        match res {
            Ok((path, bytes)) => raw_assets.insert(path, bytes),
            Err(e) => return Err(e),
        };
    }

    Ok(raw_assets)
}

#[cfg(all(target_arch = "wasm32", feature = "reqwest"))]
async fn load_urls(paths: HashSet<PathBuf>) -> Result<RawAssets> {
    let mut raw_assets = RawAssets::new();

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
                .map_err(|e| {
                    Error::FailedLoadingUrlWithReqwest(path.to_str().unwrap().to_string(), e)
                })?
                .bytes()
                .await
                .map_err(|e| {
                    Error::FailedLoadingUrlWithReqwest(path.to_str().unwrap().to_string(), e)
                })?
                .to_vec();

            #[cfg(target_arch = "wasm32")]
            {
                if std::str::from_utf8(&bytes[0..15])
                    .map(|r| r.starts_with("<!DOCTYPE html>"))
                    .unwrap_or(false)
                {
                    Err(Error::FailedLoadingUrl(
                        path.to_str().unwrap().to_string(),
                        std::str::from_utf8(&bytes).unwrap().to_string(),
                    ))?;
                }
            }
            raw_assets.insert(path, bytes);
        }
    }

    Ok(raw_assets)
}
#[cfg(not(feature = "reqwest"))]
async fn load_urls(paths: HashSet<PathBuf>) -> Result<RawAssets> {
    if !paths.is_empty() {
        return Err(Error::FeatureMissing("reqwest".to_string()));
    }
    Ok(RawAssets::new())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "reqwest"))]
async fn load_urls<Us>(urls: Us) -> Result<RawAssets>
where
    Us: IntoIterator<Item = PathBuf>,
{
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Semaphore;

    // connection limit per host (in the future make this configurable?)
    const CONN_PER_HOST: usize = 8;

    let mut tasks = tokio::task::JoinSet::new();
    // It might be more flexible to provide the client as an argument to this function
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .user_agent(USER_AGENT)
        .build()
        .unwrap();
    let it = urls.into_iter();
    // allocate enough space for the entire iterator
    let mut raw_assets = RawAssets::with_capacity(it.size_hint().1.unwrap_or(0));
    // A mapping of hosts to semaphores to limit connections.
    let mut host_connections = HashMap::new();

    for path in it {
        // Note: this is not a deep copy or anything. It's just cloning an Arc.
        // The underlying `client` is reused. We must clone it to move it
        // (possibly) across threads into the spawned task.
        let client = client.clone();

        let url = reqwest::Url::parse(match path.to_str() {
            Some(valid_unicode) => valid_unicode,
            None => return Err(Error::FailedParsingUrl("Bad unicode in url.".into())),
        })
        .map_err(|e| Error::FailedParsingUrl(e.to_string()))?;

        // This could technically fail since some valid urls (like `file::`) do
        // not have a valid hostname. It might be best to detect this scheme and
        // put their local paths in `local_paths` in `load_async_single`
        let host = match url.host() {
            Some(host) => host,
            None => return Err(Error::FailedParsingUrl("Invalid host.".into())),
        };

        // Clone our semaphore for this host. We can't acquire here or we await
        // here and block iteration, which isn't what we want. We must move this
        // inside the closure below and acquire a permit inside the spawned task.
        let semaphore = host_connections
            .entry(host.to_owned())
            .or_insert(Arc::new(Semaphore::new(CONN_PER_HOST)))
            .to_owned();

        // NOTE: We must not await inside this for loop (outside this task), or
        // we block iteration and stop spawning tasks. We want to spawn all
        // tasks, and only await within *spawned* tasks. This way all urls are
        // submitted as tasks immediately, although downloads will only happen
        // if permits are available for a given host.
        tasks.spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|e| Error::FailedLoadingUrl(path.to_string_lossy().into(), e))?;

            let bytes = response
                .bytes()
                .await
                .map_err(|e| Error::FailedLoadingUrl(path.to_string_lossy().into(), e))?
                .to_vec();

            Ok((path, bytes)) // _permit is released
        });
    }

    // Iterate over the `res`ults of the tasks as they complete
    while let Some(Ok(res)) = tasks.join_next().await {
        match res {
            Ok((path, bytes)) => raw_assets.insert(path, bytes),
            Err(e) => return Err(e),
        };
    }

    Ok(raw_assets)
}

/// Decode and add any data urls in `paths` to `raw_assets`
fn parse_data_urls(paths: HashSet<PathBuf>, raw_assets: &mut RawAssets) -> Result<()> {
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
        let mut loaded_image: Texture2D = loaded.deserialize(".png").unwrap();
        loaded_image.name = "default".to_owned();

        assert_eq!(loaded_data_url, loaded_image);
    }
}
