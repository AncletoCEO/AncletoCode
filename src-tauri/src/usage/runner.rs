use std::sync::Arc;
use std::time::Instant;

use super::providers::UsageProvider;
use super::{Emitter, ProviderUsage, UsageEvent};

pub fn collect_parallel(
    providers: &[Box<dyn UsageProvider>],
    emit: &Emitter,
) -> Vec<ProviderUsage> {
    let ids: Vec<&'static str> = providers.iter().map(|provider| provider.id()).collect();
    let labels: Vec<&'static str> = providers.iter().map(|provider| provider.label()).collect();
    let mut collected: Vec<(usize, ProviderUsage)> = std::thread::scope(|scope| {
        let handles: Vec<_> = providers
            .iter()
            .enumerate()
            .map(|(index, provider)| {
                let emit = Arc::clone(emit);
                let id = provider.id();
                scope.spawn(move || {
                    let started = Instant::now();
                    let mut usage = provider.collect();
                    usage.duration_ms = Some(started.elapsed().as_millis() as u64);
                    emit(UsageEvent::Provider {
                        index,
                        usage: usage.clone(),
                    });
                    (index, id, usage)
                })
            })
            .collect();
        handles
            .into_iter()
            .enumerate()
            .map(|(slot, handle)| match handle.join() {
                Ok(value) => (value.0, value.2),
                Err(_) => (
                    slot,
                    ProviderUsage::failed(
                        ids[slot],
                        labels[slot],
                        "El proveedor falló de forma inesperada.",
                    ),
                ),
            })
            .collect()
    });
    collected.sort_by_key(|(index, _)| *index);
    collected.into_iter().map(|(_, usage)| usage).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    use crate::usage::UsageSnapshot;

    struct FakeProvider {
        id: &'static str,
        delay_ms: u64,
        fail_panic: bool,
    }

    impl UsageProvider for FakeProvider {
        fn id(&self) -> &'static str {
            self.id
        }

        fn collect(&self) -> ProviderUsage {
            std::thread::sleep(Duration::from_millis(self.delay_ms));
            if self.fail_panic {
                panic!("proveedor roto");
            }
            ProviderUsage::failed(self.id, self.id, "fake")
        }
    }

    fn recorder() -> (Emitter, Arc<Mutex<Vec<UsageEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let emit: Emitter = Arc::new(move |event| {
            sink.lock().expect("events").push(event);
        });
        (emit, events)
    }

    #[test]
    fn keeps_declaration_order_when_providers_finish_out_of_order() {
        let providers: Vec<Box<dyn UsageProvider>> = vec![
            Box::new(FakeProvider {
                id: "lento",
                delay_ms: 80,
                fail_panic: false,
            }),
            Box::new(FakeProvider {
                id: "rapido",
                delay_ms: 5,
                fail_panic: false,
            }),
        ];
        let (emit, events) = recorder();
        let result = collect_parallel(&providers, &emit);
        assert_eq!(
            result
                .iter()
                .map(|usage| usage.provider.as_str())
                .collect::<Vec<_>>(),
            vec!["lento", "rapido"]
        );
        let emitted: Vec<String> = events
            .lock()
            .expect("events")
            .iter()
            .filter_map(|event| match event {
                UsageEvent::Provider { usage, .. } => Some(usage.provider.clone()),
                UsageEvent::Done { .. } => None,
            })
            .collect();
        assert_eq!(emitted, vec!["rapido", "lento"]);
    }

    #[test]
    fn a_panicking_provider_does_not_break_the_others() {
        let providers: Vec<Box<dyn UsageProvider>> = vec![
            Box::new(FakeProvider {
                id: "roto",
                delay_ms: 0,
                fail_panic: true,
            }),
            Box::new(FakeProvider {
                id: "sano",
                delay_ms: 0,
                fail_panic: false,
            }),
        ];
        let (emit, _events) = recorder();
        let result = collect_parallel(&providers, &emit);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].provider, "roto");
        assert_eq!(result[0].status, "error");
        assert_eq!(result[1].provider, "sano");
    }

    #[test]
    fn records_duration_per_provider() {
        let providers: Vec<Box<dyn UsageProvider>> = vec![Box::new(FakeProvider {
            id: "opencode",
            delay_ms: 30,
            fail_panic: false,
        })];
        let (emit, _events) = recorder();
        let result = collect_parallel(&providers, &emit);
        assert!(result[0].duration_ms.unwrap_or_default() >= 25);
    }

    #[test]
    fn emits_done_only_from_the_service() {
        let snapshot = UsageSnapshot {
            providers: Vec::new(),
            checked_at: 1,
            stale: false,
            refreshing: false,
            duration_ms: None,
        };
        let (emit, events) = recorder();
        emit(UsageEvent::Done { snapshot });
        assert!(matches!(
            events.lock().expect("events").first(),
            Some(UsageEvent::Done { .. })
        ));
    }
}
