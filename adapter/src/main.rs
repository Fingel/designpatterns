struct AuditEvent {
    user: String,
    action: String,
}

trait AuditSink {
    fn record(&mut self, event: &AuditEvent) -> Result<(), String>;
}

#[derive(Debug, Default, PartialEq, Eq)]
struct WhRequest(String, String);

#[derive(Debug, Default)]
struct LegacyWebhookClient {
    next_status: u16,
    requests: Vec<WhRequest>,
}

impl LegacyWebhookClient {
    fn set_next(&mut self, status: u16) {
        self.next_status = status;
    }

    fn post(&mut self, topic: &str, body: &str) -> u16 {
        self.requests
            .push(WhRequest(topic.to_string(), body.to_string()));
        self.next_status
    }
}

#[derive(Debug, Default)]
struct WebhookAuditAdapter(LegacyWebhookClient);

impl AuditSink for WebhookAuditAdapter {
    fn record(&mut self, event: &AuditEvent) -> Result<(), String> {
        let user = &event.user;
        let action = &event.action;
        let body = format!("user={user} action={action}");
        let topic = "security-audit";
        match self.0.post(topic, &body) {
            200..=299 => Ok(()),
            other => Err(format!("Error: status code {other}")),
        }
    }
}

impl WebhookAuditAdapter {
    fn into_inner(self) -> LegacyWebhookClient {
        self.0
    }

    fn new(client: LegacyWebhookClient) -> Self {
        Self(client)
    }
}

fn record_login<S: AuditSink>(sink: &mut S, user: &str) -> Result<(), String> {
    let event = AuditEvent {
        user: user.to_string(),
        action: "login".to_string(),
    };
    sink.record(&event)
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_record_login_works() {
        let mut client = LegacyWebhookClient::default();
        client.set_next(200);
        let mut adapter = WebhookAuditAdapter::new(client);
        record_login(&mut adapter, "austin").unwrap();
        assert_eq!(
            adapter.into_inner().requests[0],
            WhRequest(
                "security-audit".to_string(),
                "user=austin action=login".to_string()
            )
        );
    }

    #[test]
    fn test_record_login_ok() {
        let mut client = LegacyWebhookClient::default();
        client.set_next(201);
        let mut adapter = WebhookAuditAdapter::new(client);
        let result = record_login(&mut adapter, "austin");
        assert!(result.is_ok());
    }

    #[test]
    fn test_record_login_err() {
        let mut client = LegacyWebhookClient::default();
        client.set_next(404);
        let mut adapter = WebhookAuditAdapter::new(client);
        let result = record_login(&mut adapter, "austin");
        assert!(result.is_err());
    }
}
