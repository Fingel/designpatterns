trait InferenceRunner {
    fn backend_name(&self) -> &'static str;
    fn infer(&self, prompt: &str) -> Result<String, String>;
}

struct LocalRunner {
    model: String,
}

impl InferenceRunner for LocalRunner {
    fn backend_name(&self) -> &'static str {
        "smol-model"
    }

    fn infer(&self, prompt: &str) -> Result<String, String> {
        let name = self.model;
        Ok(format!("local[{name}]: response to \"Explain ownership\""))
    }
}

struct HostedRunner {
    model: String,
    endpoint: String,
}

impl InferenceRunner for HostedRunner {
    fn backend_name(&self) -> &'static str {
        "large-model"
    }

    fn infer(&self, prompt: &str) -> Result<String, String> {
        let name = self.model;
        let endpoint = self.endpoint;
        Ok(format!("hosted[{name} via {endpoint}]: response to \"Explain ownership\""))
    }
}

fn main() {
    println!("Hello, world!");
}
