use std::sync::Arc;

use ancleto_code_lib::usage::{default_providers, UsageEvent, UsageService};

#[test]
#[ignore = "consulta proveedores reales; ejecutar con --ignored --nocapture"]
fn measures_real_provider_latency() {
    let dir = std::env::temp_dir().join(format!("ancleto-usage-latency-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let service = UsageService::new(&dir, default_providers());
    assert!(service.begin_refresh());
    let emit = Arc::new(|event: UsageEvent| match event {
        UsageEvent::Provider { usage, .. } => eprintln!(
            "provider {}: {} ms ({})",
            usage.provider,
            usage.duration_ms.unwrap_or_default(),
            usage.status
        ),
        UsageEvent::Done { snapshot } => eprintln!(
            "total: {} ms, {} proveedores",
            snapshot.duration_ms.unwrap_or_default(),
            snapshot.providers.len()
        ),
    });
    service.refresh(emit);
    let cached = std::fs::metadata(ancleto_code_lib::usage::cache_path(&dir))
        .expect("caché escrita")
        .len();
    eprintln!("cache: {cached} bytes");
    let reopen = std::time::Instant::now();
    let cached_service = UsageService::new(&dir, default_providers());
    let opened = reopen.elapsed();
    let snapshot_start = std::time::Instant::now();
    let cached_snapshot = cached_service.snapshot(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
    );
    eprintln!(
        "cache abierta: {:?} + snapshot {:?}, refrescando={}, vencida={}",
        opened,
        snapshot_start.elapsed(),
        cached_snapshot.refreshing,
        cached_snapshot.stale
    );
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let snapshot = service.snapshot(now);
    assert!(!snapshot.providers.is_empty());
    assert!(!snapshot.stale);
    let _ = std::fs::remove_dir_all(&dir);
}
