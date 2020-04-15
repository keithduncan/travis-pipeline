use std::{
	collections::HashMap as Map,
	convert::From,
};

use serde::Serialize;

use super::travis::Travis;

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct Buildkite {
	steps: Vec<Step>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct Step {
	commands: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	label: Option<String>,
	#[serde(skip_serializing_if = "Map::is_empty")]
	agents: Map<String, String>,
	#[serde(skip_serializing_if = "Map::is_empty")]
	env: Map<String, String>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	soft_fail: Vec<Map<String, String>>,
}

fn env_for_travis_env(travis: &str) -> Map<String, String> {
	shellwords::split(travis)
		.expect("env parses")
		.into_iter()
		.map(|word| {
			let key_value: Vec<_> = word.splitn(2, '=').collect();
			(key_value[0].to_string(), key_value[1].to_string())
		})
		.collect()
}

impl From<Travis> for Buildkite {
	fn from(travis: Travis) -> Buildkite {
		let mut steps: Vec<Step> = Vec::new();

		for (rust, env) in iproduct!(travis.rust, travis.env) {
			let mut step = Step {
				commands: travis.script.clone(),
				label: Some(format!(":rust: {}, {}", rust, env)),
				agents: vec![
						("rust".to_string(), rust.clone()),
					]
					.into_iter()
					.collect(),
				env: env_for_travis_env(&env),
				soft_fail: Vec::new(),
			};

			if let Some(ref matrix) = travis.matrix {
				if let Some(allow) = matrix.get("allow_failures") {
					let optional = allow
						.iter()
						.any(|case| {
							// TODO make this generic for the iproduct fields
							if let Some(case_rust) = case.get("rust") {
								if case_rust != &rust {
									return false;
								}
							}

							if let Some(case_env) = case.get("env") {
								if case_env != &env {
									return false;
								}
							}

							true
						});

					if optional {
						step.soft_fail = vec![
							vec![
								("exit_status".to_string(), "*".to_string()),
							]
							.into_iter()
							.collect()
						];
					}
				}
			}

			steps.push(step);
		}

		Buildkite {
			steps: steps,
		}
	}
}

pub fn pipeline_for_travis_config(travis: Travis, agent_query_rules: Option<Vec<&str>>) -> Buildkite {
    let mut buildkite: Buildkite = travis.into();

    buildkite
		.steps
		.iter_mut()
		.for_each(|step| {
			if let Some(agent_query_rules) = agent_query_rules.clone() {
				let kv_rules = agent_query_rules
					.iter()
					.map(|rule| {
						let key_value: Vec<_> = rule.splitn(2, '=').collect();
						(key_value[0].to_string(), key_value[1].to_string())
					});
				step.agents.extend(kv_rules);
			}
    	});

    return buildkite;
}
