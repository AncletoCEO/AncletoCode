use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::cache::{self, CachedSnapshot};
use super::providers::UsageProvider;
use super::runner::collect_parallel;
use super::{now_secs, UsageEvent, UsageSnapshot};

pub type Emitter = Arc<dyn Fn(UsageEvent) + Send + Sync>;

struct State {
    snapshot: Option<CachedSnapshot>,
    duration_ms: Option<u64>,
}

pub struct UsageService {
    providers: Vec<Box<dyn UsageProvider>>,
    cache_path: PathBuf,
    state: Mutex<State>,
    refreshing: AtomicBool,
}

impl UsageService {
    pub fn new(config_dir: &Path, providers: Vec<Box<dyn UsageProvider>>) -> Self {
        let cache_path = cache::cache_path(config_dir);
        let snapshot = cache::read(&cache_path);
        Self {
            providers,
            cache_path,
            state: Mutex::new(State {
                snapshot,
                duration_ms: None,
            }),
            refreshing: AtomicBool::new(false),
        }
    }

    pub fn snapshot(&self, now: u64) -> UsageSnapshot {
        let state = self.lock();
        let checked_at = state
            .snapshot
            .as_ref()
            .map_or(0, |snapshot| snapshot.checked_at);
        UsageSnapshot {
            providers: state
                .snapshot
                .as_ref()
                .map_or_else(Vec::new, |snapshot| snapshot.providers.clone()),
            checked_at,
            stale: cache::is_stale(checked_at, now),
            refreshing: self.is_refreshing(),
            duration_ms: state.duration_ms,
        }
    }

    pub fn begin_refresh(&self) -> bool {
        self.refreshing
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn is_refreshing(&self) -> bool {
        self.refreshing.load(Ordering::SeqCst)
    }

    pub fn refresh(&self, emit: Emitter) {
        struct RefreshGuard<'a>(&'a AtomicBool);
        impl Drop for RefreshGuard<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::SeqCst);
            }
        }

        let _guard = RefreshGuard(&self.refreshing);
        let started = Instant::now();
        let providers = collect_parallel(&self.providers, &emit);
        let checked_at = now_secs();
        let duration_ms = started.elapsed().as_millis() as u64;
        let cached = CachedSnapshot::new(checked_at, providers);
        {
            let mut state = self.lock();
            state.snapshot = Some(cached.clone());
            state.duration_ms = Some(duration_ms);
        }
        let _ = cache::write(&self.cache_path, &cached);
        emit(UsageEvent::Done {
            snapshot: self.snapshot(now_secs()),
        });
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    use crate::usage::{cache_path, ProviderUsage, UsageMetric};

    struct FakeProvider {
        id: &'static str,
        delay_ms: u64,
    }

    impl UsageProvider for FakeProvider {
        fn id(&self) -> &'static str {
            self.id
        }

        fn collect(&self) -> ProviderUsage {
            std::thread::sleep(Duration::from_millis(self.delay_ms));
            ProviderUsage {
                provider: self.id.to_owned(),
                label: self.id.to_owned(),
                status: "ready".to_owned(),
                message: None,
                source: Some("fake".to_owned()),
                cost: Some(1.0),
                tokens: Some(10),
                sessions: Some(1),
                details: vec![UsageMetric {
                    label: "Input".to_owned(),
                    value: "10".to_owned(),
                }],
                windows: Vec::new(),
                checked_at: now_secs(),
                duration_ms: None,
            }
        }
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ancleto-usage-service-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn recorder() -> (Emitter, Arc<Mutex<Vec<UsageEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        (
            Arc::new(move |event| sink.lock().expect("events").push(event)),
            events,
        )
    }

    #[test]
    fn starts_empty_without_cache() {
        let dir = temp_dir("empty");
        let service = UsageService::new(&dir, Vec::new());
        let snapshot = service.snapshot(now_secs());
        assert!(snapshot.providers.is_empty());
        assert_eq!(snapshot.checked_at, 0);
        assert!(snapshot.stale);
        assert!(!snapshot.refreshing);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn loads_and_persists_cache() {
        let dir = temp_dir("persist");
        let providers: Vec<Box<dyn UsageProvider>> = vec![Box::new(FakeProvider {
            id: "opencode",
            delay_ms: 0,
        })];
        let service = UsageService::new(&dir, providers);
        assert!(service.begin_refresh());
        let (emit, events) = recorder();
        service.refresh(emit);
        assert!(!service.is_refreshing());
        assert!(cache_path(&dir).is_file());
        assert!(matches!(
            events.lock().expect("events").last(),
            Some(UsageEvent::Done { .. })
        ));

        assert!(service.snapshot(now_secs()).duration_ms.is_some());

        let reopened = UsageService::new(&dir, Vec::new());
        let snapshot = reopened.snapshot(now_secs());
        assert_eq!(snapshot.providers.len(), 1);
        assert!(!snapshot.stale);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn allows_only_one_concurrent_refresh() {
        let dir = temp_dir("single-flight");
        let service = UsageService::new(&dir, Vec::new());
        assert!(service.begin_refresh());
        assert!(!service.begin_refresh());
        assert!(service.is_refreshing());
        let (emit, _events) = recorder();
        service.refresh(emit);
        assert!(!service.is_refreshing());
        assert!(service.begin_refresh());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn serves_the_previous_snapshot_while_refreshing() {
        let dir = temp_dir("serving");
        let providers: Vec<Box<dyn UsageProvider>> = vec![Box::new(FakeProvider {
            id: "lento",
            delay_ms: 120,
        })];
        let service = Arc::new(UsageService::new(&dir, providers));
        assert!(service.begin_refresh());
        let (emit, _events) = recorder();
        let worker = Arc::clone(&service);
        let handle = std::thread::spawn(move || worker.refresh(emit));
        std::thread::sleep(Duration::from_millis(20));
        let during = service.snapshot(now_secs());
        assert!(during.refreshing);
        assert!(during.providers.is_empty());
        handle.join().expect("refresh");
        let after = service.snapshot(now_secs());
        assert!(!after.refreshing);
        assert_eq!(after.providers.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn marks_stale_cache_by_ttl() {
        let dir = temp_dir("stale");
        let old = CachedSnapshot::new(1, Vec::new());
        cache::write(&cache_path(&dir), &old).expect("write");
        let service = UsageService::new(&dir, Vec::new());
        assert!(service.snapshot(now_secs()).stale);
        let _ = fs::remove_dir_all(&dir);
    }
}
