//! Pictures for the interface: covers, backgrounds and logos from local files or the stores'
//! CDNs. Downloads go to a cache folder; decoding and shrinking happen off the UI thread, so the
//! window only ever gets small, ready pixels. Kept small on purpose: a cover is held at shelf
//! size, not at the size the store serves it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use slint::{Rgba8Pixel, SharedPixelBuffer};

/// How big a picture is kept: covers at shelf size, backgrounds at screen size.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fit {
    Cover,
    Hero,
    Logo,
}

impl Fit {
    fn max(self) -> (u32, u32) {
        match self {
            Fit::Cover => (300, 450),
            Fit::Hero => (1920, 1080),
            Fit::Logo => (800, 400),
        }
    }
}

pub type Pixels = SharedPixelBuffer<Rgba8Pixel>;

struct Loader {
    dir: PathBuf,
    done: Mutex<HashMap<(String, Fit), Option<Pixels>>>,
    gate: tokio::sync::Semaphore,
    http: reqwest::Client,
}

static LOADER: OnceLock<Arc<Loader>> = OnceLock::new();

pub fn init(cache_dir: PathBuf) {
    let _ = std::fs::create_dir_all(&cache_dir);
    let http = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).user_agent("Mantel (game launcher)").build().unwrap_or_default();
    let _ = LOADER.set(Arc::new(Loader { dir: cache_dir, done: Mutex::new(HashMap::new()), gate: tokio::sync::Semaphore::new(4), http }));
}

/// Forgets decoded pictures, e.g. backgrounds no longer on screen.
pub fn forget(fit: Fit) {
    if let Some(l) = LOADER.get() {
        l.done.lock().unwrap().retain(|(_, f), _| *f != fit);
    }
}

/// A picture already decoded, if it is.
pub fn ready(src: &str, fit: Fit) -> Option<Option<Pixels>> {
    LOADER.get()?.done.lock().unwrap().get(&(src.to_string(), fit)).cloned()
}

/// Decodes the first of `srcs` that works and hands it to `done` (on a worker thread).
/// Nothing at all is handed over when none of them does.
pub fn load(srcs: Vec<String>, fit: Fit, done: impl FnOnce(String, Pixels) + Send + 'static) {
    let Some(loader) = LOADER.get().cloned() else { return };
    crate::runtime().spawn(async move {
        for src in srcs {
            if let Some(hit) = loader.done.lock().unwrap().get(&(src.clone(), fit)).cloned() {
                match hit {
                    Some(px) => return done(src, px),
                    None => continue,
                }
            }
            let _permit = loader.gate.acquire().await;
            let file = match local(&loader, &src).await {
                Some(f) => f,
                None => {
                    loader.done.lock().unwrap().insert((src.clone(), fit), None);
                    continue;
                }
            };
            let decoded = tokio::task::spawn_blocking(move || decode(&file, fit)).await.ok().flatten();
            loader.done.lock().unwrap().insert((src.clone(), fit), decoded.clone());
            if let Some(px) = decoded {
                return done(src, px);
            }
        }
    });
}

fn cache_name(url: &str) -> String {
    let h = url.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3));
    let ext = url.rsplit('.').next().filter(|e| e.len() <= 4 && e.chars().all(|c| c.is_ascii_alphanumeric())).unwrap_or("img");
    format!("{h:016x}.{ext}")
}

/// A file to decode: the path itself, or the cached download of a URL.
async fn local(loader: &Loader, src: &str) -> Option<PathBuf> {
    if !src.starts_with("http://") && !src.starts_with("https://") {
        let p = PathBuf::from(src);
        return p.is_file().then_some(p);
    }
    let file = loader.dir.join(cache_name(src));
    if file.is_file() {
        return Some(file);
    }
    let resp = loader.http.get(src).send().await.ok()?.error_for_status().ok()?;
    let bytes = resp.bytes().await.ok()?;
    if bytes.is_empty() || bytes.len() > 25 * 1024 * 1024 {
        return None;
    }
    let tmp = file.with_extension("part");
    std::fs::write(&tmp, &bytes).ok()?;
    std::fs::rename(&tmp, &file).ok()?;
    Some(file)
}

fn decode(file: &Path, fit: Fit) -> Option<Pixels> {
    let ext = file.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    if ext == "ico" || ext == "svg" {
        return None;
    }
    let img = image::ImageReader::open(file).ok()?.with_guessed_format().ok()?.decode().ok()?;
    let (mw, mh) = fit.max();
    let img = if img.width() > mw || img.height() > mh { img.resize(mw, mh, image::imageops::FilterType::Triangle) } else { img };
    let rgba = img.to_rgba8();
    Some(SharedPixelBuffer::clone_from_slice(rgba.as_raw(), rgba.width(), rgba.height()))
}
