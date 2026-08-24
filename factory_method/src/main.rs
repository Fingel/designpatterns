/// GOF factory method
/// create_runner has behavior independant from the backend itself
/// useful for validation. Different environments can provide different
/// implementations.
///
/// For a small set like this an enum would probably be better - this
/// is just to demonstrate the pattern
trait InferenceRunner {
    fn backend_name(&self) -> &'static str;
    fn infer(&self, prompt: &str) -> Result<String, String>;
}

struct LocalRunner {
    model: String,
}

impl InferenceRunner for LocalRunner {
    fn backend_name(&self) -> &'static str {
        "local"
    }

    fn infer(&self, prompt: &str) -> Result<String, String> {
        let name = &self.model;
        let backend_name = &self.backend_name();
        Ok(format!("{backend_name}[{name}]: response to {prompt}"))
    }
}

struct HostedRunner {
    model: String,
    endpoint: String,
}

impl InferenceRunner for HostedRunner {
    fn backend_name(&self) -> &'static str {
        "hosted"
    }

    fn infer(&self, prompt: &str) -> Result<String, String> {
        let name = &self.model;
        let endpoint = &self.endpoint;
        let backend_name = &self.backend_name();
        Ok(format!(
            "{backend_name}[{name} via {endpoint}]: response to {prompt}"
        ))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct EvaluationReport {
    pub backend: String,
    pub prompt: String,
    pub response: String,
}

trait EvaluationEnvironment {
    type Runner: InferenceRunner;

    fn create_runner(&self, model: &str) -> Result<Self::Runner, String>;

    fn evaluate(&self, model: &str, prompt: &str) -> Result<EvaluationReport, String> {
        if model.trim().is_empty() {
            return Err("Model must not be empty".to_string());
        }
        if prompt.trim().is_empty() {
            return Err("Prompt must not be empty".to_string());
        }

        let runner = self.create_runner(model)?;
        let response = runner.infer(prompt)?;

        Ok(EvaluationReport {
            backend: runner.backend_name().to_string(),
            prompt: prompt.to_string(),
            response,
        })
    }
}

struct LocalEnvironment;

impl EvaluationEnvironment for LocalEnvironment {
    type Runner = LocalRunner;

    fn create_runner(&self, model: &str) -> Result<LocalRunner, String> {
        Ok(LocalRunner {
            model: model.to_string(),
        })
    }
}

struct HostedEnvironment {
    endpoint: String,
}

impl EvaluationEnvironment for HostedEnvironment {
    type Runner = HostedRunner;

    fn create_runner(&self, model: &str) -> Result<HostedRunner, String> {
        if self.endpoint.is_empty() {
            Err("Endpoint must not be empty".to_string())
        } else {
            Ok(HostedRunner {
                model: model.to_string(),
                endpoint: self.endpoint.clone(),
            })
        }
    }
}

fn run_evaluation<E>(environment: &E, model: &str, prompt: &str) -> Result<(), String>
where
    E: EvaluationEnvironment,
{
    let report = environment.evaluate(model, prompt)?;
    println!("{report:#?}");
    Ok(())
}

fn main() {
    let backend = std::env::var("INFERENCE_BACKEND").unwrap_or_else(|_| "local".to_string());

    match backend.as_str() {
        "local" => run_evaluation(&LocalEnvironment, "smol-model", "Explain Rust Ownership"),
        "hosted" => run_evaluation(
            &HostedEnvironment {
                endpoint: "https://example.com".to_string(),
            },
            "large-model",
            "Explain Rust Ownership",
        ),
        other => Err(format!("unknown backend: {other}")),
    }
    .unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_local_env_creates_local_runner() {
        let environment = LocalEnvironment;
        let runner = environment.create_runner("test-model").unwrap();
        assert_eq!(runner.backend_name(), "local");
    }

    #[test]
    fn hosted_environment_passes_its_endpoint_to_the_runner() {
        let environment = HostedEnvironment {
            endpoint: "https://test.example".to_string(),
        };
        let runner = environment.create_runner("test-model").unwrap();
        assert_eq!(runner.endpoint, "https://test.example");
    }

    #[test]
    fn test_evaluate_correct_report() {
        let environment = LocalEnvironment;
        let prompt = "Explain Rust Ownership";
        let report = environment.evaluate("tiny-model", prompt).unwrap();
        assert_eq!(report.backend, "local");
        assert_eq!(report.prompt, "Explain Rust Ownership");
        assert!(report.response.contains(&format!("response to {prompt}")));
    }

    #[test]
    fn test_empty_prompt_rejected() {
        let environment = LocalEnvironment;
        let prompt = "";
        let report = environment.evaluate("tiny-model", prompt);
        assert_eq!(report, Err("Prompt must not be empty".to_string()));

        let environment = HostedEnvironment {
            endpoint: "http://example.com".to_string(),
        };
        let prompt = "";
        let report = environment.evaluate("large-model", prompt);
        assert_eq!(report, Err("Prompt must not be empty".to_string()));
    }
}
