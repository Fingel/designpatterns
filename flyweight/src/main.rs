use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct StringInterner {
    entries: HashMap<String, Weak<str>>,
}

impl StringInterner {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn intern(&mut self, value: &str) -> Arc<str> {
        if let Some(shared) = self.entries.get(value).and_then(Weak::upgrade) {
            return shared;
        }

        let shared: Arc<str> = Arc::from(value);

        self.entries
            .insert(value.to_string(), Arc::downgrade(&shared));

        shared
    }

    fn prune(&mut self) {
        self.entries.retain(|_key, value| value.strong_count() > 0);
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

struct LogEvent {
    service: Arc<str>,
    level: Arc<str>,
    timestamp: u64,
    message: String,
}

impl LogEvent {
    fn new(message: &str, service: &str, level: &str, registry: &mut StringInterner) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward")
            .as_secs();
        let service = registry.intern(service);
        let level = registry.intern(level);
        let message = message.to_string();
        Self {
            service,
            level,
            timestamp,
            message,
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_same_string() {
        let mut interner = StringInterner::new();
        let first = interner.intern("foo");
        let second = interner.intern("foo");
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn test_intern_different_string() {
        let mut interner = StringInterner::new();
        let first = interner.intern("foo");
        let second = interner.intern("bar");
        assert!(!Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn test_logevent() {
        let mut registry = StringInterner::new();
        let first = LogEvent::new("test message 1", "test-service", "warn", &mut registry);
        let second = LogEvent::new("test message 2", "test-service", "warn", &mut registry);
        assert!(Arc::ptr_eq(&first.service, &second.service));
        assert!(Arc::ptr_eq(&first.level, &second.level));
    }

    #[test]
    fn test_logevent_prune() {
        let mut registry = StringInterner::new();
        {
            let first = LogEvent::new("test message 1", "test-service", "warn", &mut registry);
            let second = LogEvent::new("test message 2", "test-service", "warn", &mut registry);
            assert!(Arc::ptr_eq(&first.service, &second.service));
            assert!(Arc::ptr_eq(&first.level, &second.level));
            let third  = LogEvent::new("test message 3", "other-service", "warn", &mut registry);
            assert_eq!(registry.len(), 3);
        }
        registry.prune();
        let third = LogEvent::new("test message 4", "new-service", "error", &mut registry);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_reuse() {
        let mut registry = StringInterner::new();
        let first = registry.intern("test");
        let weak = Arc::downgrade(&first);
        drop(first);
        assert!(weak.upgrade().is_none());
        registry.prune();
        let second = registry.intern("test");
        assert_eq!(second.as_ref(), "test");
    }

}
