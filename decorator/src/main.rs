trait Log {
    fn write(&mut self, message: &str) -> Result<(), String>;
}

struct MemoryLog {
    messages: Vec<String>,
}

impl Log for MemoryLog {
    fn write(&mut self, message: &str) -> Result<(), String> {
        self.messages.push(message.to_string());
        Ok(())
    }
}

impl MemoryLog {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

struct UppercaseLog<L: Log> {
    inner: L,
}

impl<L: Log> Log for UppercaseLog<L> {
    fn write(&mut self, message: &str) -> Result<(), String> {
        let message = message.to_uppercase();
        self.inner.write(&message)
    }
}

impl<L: Log> UppercaseLog<L> {
    fn new(wraps: L) -> Self {
        Self { inner: wraps }
    }

    fn into_inner(self) -> L {
        self.inner
    }
}

struct ContextLog<L: Log> {
    inner: L,
    service: String,
}

impl<L: Log> ContextLog<L> {
    fn new(wraps: L, service: &str) -> Self {
        Self {
            inner: wraps,
            service: service.to_string(),
        }
    }

    fn into_inner(self) -> L {
        self.inner
    }
}

impl<L: Log> Log for ContextLog<L> {
    fn write(&mut self, message: &str) -> Result<(), String> {
        let message = format!("[service={}] {}", self.service, message);
        self.inner.write(&message)
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memorylog_stores_message() {
        let mut mem_log = MemoryLog::new();
        mem_log.write("test log").unwrap();
        assert_eq!(mem_log.messages[0], "test log");
    }

    #[test]
    fn test_contextlog() {
        let mem_log = MemoryLog::new();
        let mut ctx_log = ContextLog::new(mem_log, "test-service");
        ctx_log.write("test log").unwrap();
        let mem_log = ctx_log.into_inner();
        assert_eq!(mem_log.messages[0], "[service=test-service] test log");
    }

    #[test]
    fn test_upperlog() {
        let mem_log = MemoryLog::new();
        let mut upp_log = UppercaseLog::new(mem_log);
        upp_log.write("test log").unwrap();
        let mem_log = upp_log.into_inner();
        assert_eq!(mem_log.messages[0], "TEST LOG");
    }

    #[test]
    fn test_both() {
        let mem_log = MemoryLog::new();
        let upp_log = UppercaseLog::new(mem_log);
        let mut ctx_log = ContextLog::new(upp_log, "test-service");

        ctx_log.write("test log").unwrap();
        let mem_log = ctx_log.into_inner().into_inner();
        assert_eq!(mem_log.messages[0], "[SERVICE=TEST-SERVICE] TEST LOG");
    }
}
