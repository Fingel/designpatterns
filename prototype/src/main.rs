use std::collections::HashMap;

/// Prototype
/// Creates a registry of pre-configured profiles that implement clone
/// The registry itself is a hashmap

#[derive(Debug, Clone)]
struct AgentProfile {
    model: String,
    system_prompt: String,
    temperature: f32,
    tools: Vec<String>,
    task_id: Option<String>,
}

#[derive(Debug, Default)]
struct AgentRegistry {
    prototypes: HashMap<String, AgentProfile>,
}

impl AgentRegistry {
    fn register(&mut self, name: String, prototype: AgentProfile) {
        self.prototypes.insert(name, prototype);
    }

    fn instantiate(&self, name: &str, task_id: String) -> Result<AgentProfile, String> {
        let result = self
            .prototypes
            .get(name)
            .ok_or_else(|| "could not find prototype with that name".to_string())?;
        let mut result = result.clone();
        result.task_id = Some(task_id);
        Ok(result)
    }
}

fn main() {
    println!("Hello, world!!!");
}

#[cfg(test)]
mod test {
    use super::*;

    fn profile_proto() -> AgentProfile {
        AgentProfile {
            model: "test-model".to_string(),
            system_prompt: "make me a sandwich".to_string(),
            temperature: 69.0,
            tools: vec!["read-file", "write-file", "spread-mayo"]
                .into_iter()
                .map(String::from)
                .collect(),
            task_id: None,
        }
    }

    #[test]
    fn test_instantiate() {
        let profile = profile_proto();
        let mut registry = AgentRegistry::default();
        registry.register("test-name".to_string(), profile.clone());
        let copy = registry
            .instantiate("test-name", "muh task id".to_string())
            .unwrap();
        assert_eq!(copy.model, profile.model);
        assert_eq!(copy.system_prompt, profile.system_prompt);
        assert_eq!(copy.task_id, Some("muh task id".to_string()));
        assert_eq!(profile.task_id, None);
        assert_eq!(copy.temperature, profile.temperature);
        assert_eq!(copy.tools, profile.tools);
    }

    #[test]
    fn test_tools_independant() {
        let profile = AgentProfile {
            model: "test-model".to_string(),
            system_prompt: "make me a sandwich".to_string(),
            temperature: 69.0,
            tools: vec![],
            task_id: None,
        };
        let mut registry = AgentRegistry::default();
        registry.register("test-name".to_string(), profile.clone());
        let mut copy = registry
            .instantiate("test-name", "muh task id".to_string())
            .unwrap();
        let copy2 = registry
            .instantiate("test-name", "muh task id".to_string())
            .unwrap();
        assert!(profile.tools.is_empty());
        assert!(copy.tools.is_empty());
        assert!(copy2.tools.is_empty());
        copy.tools.push("foo".to_string());
        assert!(profile.tools.is_empty());
        assert_eq!(copy.tools[0], "foo".to_string());
        assert!(copy2.tools.is_empty());
    }
}
