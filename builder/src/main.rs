/// Builder pattern
/// Defined multiple builders that output different types
/// but share a common api that a creator uses to
/// orchestrate them.

#[derive(Debug)]
struct ReleaseSpec {
    package: String,
    run_tests: bool,
    target: String,
}

trait ReleaseBuilder {
    type Output;

    fn prepare(&mut self, package: &str);
    fn run_tests(&mut self);
    fn compile(&mut self, target: &str);
    fn package(&mut self, package: &str, target: &str);
    fn finish(self) -> Self::Output;
}

#[derive(Default)]
struct ShellScriptBuilder {
    lines: Vec<String>,
}

impl ReleaseBuilder for ShellScriptBuilder {
    type Output = String;

    fn prepare(&mut self, package: &str) {
        self.lines.push("#!/usr/bin/env bash".to_string());
        self.lines.push("set -euo pipefail".to_string());
        self.lines.push(format!("echo \"Preparing {package}\""));
    }

    fn run_tests(&mut self) {
        self.lines.push("cargo test".to_string());
    }

    fn compile(&mut self, target: &str) {
        self.lines
            .push(format!("cargo build --release --target {target}"));
    }

    fn package(&mut self, package: &str, target: &str) {
        self.lines.push(format!(
            "tar -czf {package}.tar.gz target/{target}/release/{package}"
        ));
    }

    fn finish(self) -> Self::Output {
        self.lines.join("\n")
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ReleaseStep {
    Prepare { package: String },
    Test,
    Compile { target: String },
    Package { package: String, target: String },
}

#[derive(Debug, PartialEq, Eq)]
struct ExecutionPlan {
    steps: Vec<ReleaseStep>,
}

#[derive(Default)]
struct ExecutionPlanBuilder {
    steps: Vec<ReleaseStep>,
}

impl ReleaseBuilder for ExecutionPlanBuilder {
    type Output = ExecutionPlan;

    fn prepare(&mut self, package: &str) {
        self.steps.push(ReleaseStep::Prepare {
            package: package.to_string(),
        });
    }

    fn run_tests(&mut self) {
        self.steps.push(ReleaseStep::Test);
    }

    fn compile(&mut self, target: &str) {
        self.steps.push(ReleaseStep::Compile {
            target: target.to_string(),
        });
    }

    fn package(&mut self, package: &str, target: &str) {
        self.steps.push(ReleaseStep::Package {
            package: package.to_string(),
            target: target.to_string(),
        });
    }

    fn finish(self) -> Self::Output {
        ExecutionPlan { steps: self.steps }
    }
}

fn create_release<B>(spec: &ReleaseSpec, mut builder: B) -> B::Output
where
    B: ReleaseBuilder,
{
    builder.prepare(&spec.package);

    if spec.run_tests {
        builder.run_tests();
    }

    builder.compile(&spec.target);
    builder.package(&spec.package, &spec.target);

    builder.finish()
}

fn main() {
    let format = std::env::var("OUTPUT_FORMAT").unwrap_or_else(|_| "plan".to_string());

    let spec = ReleaseSpec {
        package: "payment-service".to_string(),
        run_tests: true,
        target: "x86_64-unknown-linux-musl".to_string(),
    };

    match format.as_str() {
        "shell" => {
            let output = create_release(&spec, ShellScriptBuilder::default());
            println!("{output}");
        }
        "plan" => {
            let output = create_release(&spec, ExecutionPlanBuilder::default());
            println!("{output:#?}");
        }
        other => {
            eprintln!("Unknown output format {other}");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_release_with_tests_has_all_steps_in_order() {
        let spec = ReleaseSpec {
            package: "unittest".to_string(),
            run_tests: true,
            target: "x86_64-unknown-linux-musl".to_string(),
        };
        let plan = create_release(&spec, ExecutionPlanBuilder::default());
        assert_eq!(
            plan.steps,
            vec![
                ReleaseStep::Prepare {
                    package: "unittest".to_string(),
                },
                ReleaseStep::Test,
                ReleaseStep::Compile {
                    target: "x86_64-unknown-linux-musl".to_string(),
                },
                ReleaseStep::Package {
                    package: "unittest".to_string(),
                    target: "x86_64-unknown-linux-musl".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_release_without_tests_has_all_other_steps_in_order() {
        let spec = ReleaseSpec {
            package: "unittest".to_string(),
            run_tests: false,
            target: "x86_64-unknown-linux-musl".to_string(),
        };
        let plan = create_release(&spec, ExecutionPlanBuilder::default());
        assert_eq!(
            plan.steps,
            vec![
                ReleaseStep::Prepare {
                    package: "unittest".to_string(),
                },
                ReleaseStep::Compile {
                    target: "x86_64-unknown-linux-musl".to_string(),
                },
                ReleaseStep::Package {
                    package: "unittest".to_string(),
                    target: "x86_64-unknown-linux-musl".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_shell_has_test_output() {
        let spec = ReleaseSpec {
            package: "unittest".to_string(),
            run_tests: true,
            target: "x86_64-unknown-linux-musl".to_string(),
        };
        let plan = create_release(&spec, ShellScriptBuilder::default());
        assert!(plan.contains("cargo test"));
    }

    #[test]
    fn test_shell_does_not_have_test_output() {
        let spec = ReleaseSpec {
            package: "unittest".to_string(),
            run_tests: false,
            target: "x86_64-unknown-linux-musl".to_string(),
        };
        let plan = create_release(&spec, ShellScriptBuilder::default());
        assert!(!plan.contains("cargo test"));
    }
}
