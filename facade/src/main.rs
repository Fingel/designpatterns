use std::fmt::Display;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
enum LoggerError {
    Send(mpsc::SendError<String>),
    Cleanup,
}

impl Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerError::Send(mpsc_send_error) => write!(f, "{}", mpsc_send_error),
            LoggerError::Cleanup=> write!(f, "Could not clean up logger"),
        }
    }
}

impl std::error::Error for LoggerError {}

struct BackgroundLogger {
    tx: mpsc::Sender<String>,
    handle: thread::JoinHandle<Vec<String>>,
}

impl BackgroundLogger {
    fn start() -> Self {
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || rx.into_iter().collect());
        Self { tx, handle }
    }

    fn log(&self, message: impl Into<String>) -> Result<(), LoggerError> {
        self.tx.send(message.into()).map_err(LoggerError::Send)
    }

    fn shutdown(self) -> Result<Vec<String>, LoggerError> {
        drop(self.tx);
        let logs = self.handle.join();
        logs.map_err(|_| LoggerError::Cleanup)
    }
}

fn main() {
    let logger = BackgroundLogger::start();
    logger.log("hello").unwrap();
    logger.log("world").unwrap();
    let logs = logger.shutdown().unwrap();
    println!("Hello, world!");
    println!("{:?}", logs);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_log_three() {
        let logger = BackgroundLogger::start();
        logger.log("hello").unwrap();
        logger.log("world").unwrap();
        logger.log("!").unwrap();
        let logs = logger.shutdown().unwrap();
        assert_eq!(logs, ["hello", "world", "!"])
    }

    #[test]
    fn test_empty_logger() {
        let logger = BackgroundLogger::start();
        let logs = logger.shutdown().unwrap();
        assert!(logs.is_empty());
    }

    #[test]
    fn test_into_string() {
        let logger = BackgroundLogger::start();
        logger.log("hello".to_string()).unwrap();
        logger.log("world").unwrap();
        logger.log('!').unwrap();
        let logs = logger.shutdown().unwrap();
        assert_eq!(logs, ["hello", "world", "!"])
    }
}
