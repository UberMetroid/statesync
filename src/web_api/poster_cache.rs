//! Process-local cache of proxied Primary (poster) images.
//! Keyed by server name + item id so we fetch each title once and keep it.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

const MAX_ENTRIES: usize = 256;
const TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone)]
pub struct CachedPoster {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

struct Entry {
    poster: CachedPoster,
    at: Instant,
}

static CACHE: LazyLock<Mutex<HashMap<String, Entry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn cache_key(server: &str, item_id: &str) -> String {
    format!("{}\0{}", server, item_id)
}

pub fn get(server: &str, item_id: &str) -> Option<CachedPoster> {
    let key = cache_key(server, item_id);
    let mut guard = CACHE.lock().ok()?;
    let entry = guard.get(&key)?;
    if entry.at.elapsed() > TTL {
        guard.remove(&key);
        return None;
    }
    Some(entry.poster.clone())
}

pub fn put(server: &str, item_id: &str, poster: CachedPoster) {
    let Ok(mut guard) = CACHE.lock() else {
        return;
    };
    if guard.len() >= MAX_ENTRIES {
        // Drop oldest half when full (simple eviction).
        let mut ages: Vec<(String, Instant)> =
            guard.iter().map(|(k, v)| (k.clone(), v.at)).collect();
        ages.sort_by_key(|(_, at)| *at);
        let drop_n = ages.len() / 2;
        for (k, _) in ages.into_iter().take(drop_n) {
            guard.remove(&k);
        }
    }
    guard.insert(
        cache_key(server, item_id),
        Entry {
            poster,
            at: Instant::now(),
        },
    );
}
