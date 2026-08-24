use std::cell::RefCell;
use std::rc::Rc;

/// Abstract Factory pattern using dynamic dispatch.
/// A more rust-like implementation would use generics.

type SharedEvents = Rc<RefCell<Vec<String>>>;

pub trait NotificationSender {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String>;
}

pub trait AuditLog {
    fn record(&mut self, event: &str) -> Result<(), String>;
}

pub trait ServiceFactory {
    fn create_sender(&self) -> Box<dyn NotificationSender>;
    fn create_audit_log(&self) -> Box<dyn AuditLog>;
}

struct HttpNotificationSender;

impl HttpNotificationSender {
    fn send_to_endpoint(&self, recipient: &str, message: &str) -> Result<(), String> {
        println!("Sending {message} to {recipient} via http");
        Ok(())
    }
}

impl NotificationSender for HttpNotificationSender {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String> {
        self.send_to_endpoint(recipient, message)?;
        Ok(())
    }
}

struct DatabaseAuditLog {
    events: Vec<String>,
}

impl DatabaseAuditLog {
    fn add_event_to_database(&mut self, event: &str) -> Result<(), String> {
        self.events.push(event.to_string());
        Ok(())
    }
}

impl AuditLog for DatabaseAuditLog {
    fn record(&mut self, event: &str) -> Result<(), String> {
        self.add_event_to_database(event)?;
        println!("Added {event} to database");
        Ok(())
    }
}

pub struct ProductionServiceFactory;

impl ServiceFactory for ProductionServiceFactory {
    fn create_sender(&self) -> Box<dyn NotificationSender> {
        Box::new(HttpNotificationSender)
    }

    fn create_audit_log(&self) -> Box<dyn AuditLog> {
        Box::new(DatabaseAuditLog { events: Vec::new() })
    }
}

struct ConsoleNotificationSender;

impl ConsoleNotificationSender {
    fn print_to_console(&self, recipient: &str, message: &str) -> Result<(), String> {
        println!("DEBUG: {recipient} -> {message}");
        Ok(())
    }
}

impl NotificationSender for ConsoleNotificationSender {
    fn send(&self, recipient: &str, message: &str) -> Result<(), String> {
        self.print_to_console(recipient, message)?;
        Ok(())
    }
}

struct InMemoryAuditLog {
    events: SharedEvents,
}

impl InMemoryAuditLog {
    fn add_event(&mut self, event: &str) -> Result<(), String> {
        self.events.borrow_mut().push(event.to_string());
        Ok(())
    }
}

impl AuditLog for InMemoryAuditLog {
    fn record(&mut self, event: &str) -> Result<(), String> {
        self.add_event(event)?;
        Ok(())
    }
}

struct LocalServiceFactory {
    pub audit_storage: SharedEvents,
}

impl LocalServiceFactory {
    pub fn new(events: SharedEvents) -> Self {
        LocalServiceFactory {
            audit_storage: events,
        }
    }
}

impl ServiceFactory for LocalServiceFactory {
    fn create_sender(&self) -> Box<dyn NotificationSender> {
        Box::new(ConsoleNotificationSender)
    }
    fn create_audit_log(&self) -> Box<dyn AuditLog> {
        Box::new(InMemoryAuditLog {
            events: self.audit_storage.clone(),
        })
    }
}

pub fn notify_user(
    factory: &dyn ServiceFactory,
    recipient: &str,
    message: &str,
) -> Result<(), String> {
    let sender = factory.create_sender();
    let mut audit_log = factory.create_audit_log();

    sender.send(recipient, message)?;
    audit_log.record(&format!("Notification sent to {recipient}"))?;

    Ok(())
}

fn get_factory() -> Box<dyn ServiceFactory> {
    let environment = std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());

    match environment.as_str() {
        "production" => Box::new(ProductionServiceFactory),
        _ => Box::new(LocalServiceFactory {
            audit_storage: Rc::new(RefCell::new(Vec::new())),
        }),
    }
}

fn main() {
    let factory = get_factory();
    notify_user(
        factory.as_ref(),
        "user@example.com",
        "This should appear locally",
    )
    .unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_factory() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let factory = LocalServiceFactory::new(Rc::clone(&events));
        notify_user(&factory, "user@example.com", "This should appear locally").unwrap();
        assert_eq!(
            events.borrow().as_slice(),
            ["Notification sent to user@example.com"]
        );
    }
}
