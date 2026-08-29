trait Transport {
    fn send(&mut self, subject: &str, body: &str) -> Result<(), String>;
}

#[derive(Debug, Default)]
struct ConsoleTransport {
    messages: Vec<String>,
}

impl Transport for ConsoleTransport {
    fn send(&mut self, subject: &str, body: &str) -> Result<(), String> {
        let message = format!("Subject={subject} Body={body}");
        self.messages.push(message.clone());
        println!("{}", message);
        Ok(())
    }
}

#[derive(Debug, Default)]
struct WebhookTransport {
    messages: Vec<String>,
}

impl Transport for WebhookTransport {
    fn send(&mut self, subject: &str, body: &str) -> Result<(), String> {
        let message = format!("{{\"subject\": \"{subject}\", \"body\": \"{body}\"}}");
        self.messages.push(message.clone());
        self.post(&message);
        Ok(())
    }
}

impl WebhookTransport {
    fn post(&self, msg: &str) {
        println!("beep boop sent over the network> {msg}");
    }
}

struct SecurityAlerts<T: Transport>(T);

impl<T: Transport> SecurityAlerts<T> {
    fn new(transport: T) -> Self {
        Self(transport)
    }

    fn into_inner(self) -> T {
        self.0
    }

    fn failed_login(&mut self, user: &str) -> Result<(), String> {
        let subject = "Security alert";
        let body = format!("Failed login for {user}");
        self.0.send(subject, &body)
    }
}

struct DeploymentAlerts<T: Transport>(T);

impl<T: Transport> DeploymentAlerts<T> {
    fn new(transport: T) -> Self {
        Self(transport)
    }

    fn into_inner(self) -> T {
        self.0
    }

    fn completed(&mut self, service: &str, version: &str) -> Result<(), String> {
        let subject = "Deployment completed";
        let body = format!("{service} version {version} was deployed");
        self.0.send(subject, &body)
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sec_alert_console() {
        let transport = ConsoleTransport::default();
        let mut alert = SecurityAlerts::new(transport);
        alert.failed_login("austin").unwrap();
        let messages = alert.into_inner().messages;
        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert_eq!(
            message,
            "Subject=Security alert Body=Failed login for austin"
        );
    }

    #[test]
    fn test_sec_alert_webbook() {
        let transport = WebhookTransport::default();
        let mut alert = SecurityAlerts::new(transport);
        alert.failed_login("austin").unwrap();
        let messages = alert.into_inner().messages;
        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert_eq!(
            message,
            "{\"subject\": \"Security alert\", \"body\": \"Failed login for austin\"}"
        );
    }

    #[test]
    fn test_deploy_console() {
        let transport = ConsoleTransport::default();
        let mut alert = DeploymentAlerts::new(transport);
        alert.completed("test", "1.0.0").unwrap();
        let messages = alert.into_inner().messages;
        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert_eq!(
            message,
            "Subject=Deployment completed Body=test version 1.0.0 was deployed"
        );
    }

    #[test]
    fn test_deploy_webhook() {
        let transport = WebhookTransport::default();
        let mut alert = DeploymentAlerts::new(transport);
        alert.completed("test", "1.0.0").unwrap();
        let messages = alert.into_inner().messages;
        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert_eq!(
            message,
            "{\"subject\": \"Deployment completed\", \"body\": \"test version 1.0.0 was deployed\"}"
        );
    }
}
