use std::{
	collections::HashMap as Map,
	convert::From,
};

use serde::Serialize;

use super::travis::{self, Travis};

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
				agents: {
					let mut agents = Map::new();

					if let Some(image) = rust.parse::<travis::Rust>().ok().and_then(|rust| rust.image()) {
						agents.insert("image".to_string(), image);
					} else {
						eprintln!("cannot generate step for rust: {:?}, env: {:?}", rust, env);
						continue;
					}

					agents
				},
				env: env_for_travis_env(&env),
				soft_fail: Vec::new(),
			};

			if let Some(allow) = travis.matrix.as_ref().and_then(|matrix| matrix.get("allow_failures")) {
				let optional = allow
					.iter()
					.any(|ref case| {
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

#[cfg(test)]
mod tests {
	use super::{
		Map,
		Travis,
		Buildkite,
		Step,
	};

	use pretty_assertions::assert_eq;

	#[test]
    fn convert_env() {
    	let travis_env = "CRATE=boards/feather_m4 EXAMPLES=\"--example=blinky_basic --example=blinky_rtfm\"";
    	println!("{:#?}", travis_env);
    	
    	let buildkite_env = super::env_for_travis_env(travis_env);
    	println!("{:#?}", buildkite_env);

    	assert_eq!(buildkite_env, vec![
    		("CRATE".to_string(), "boards/feather_m4".to_string()),
    		("EXAMPLES".to_string(), "--example=blinky_basic --example=blinky_rtfm".to_string()),
    	]
    	.into_iter()
    	.collect::<Map<_, _>>());
    }

    #[test]
    fn translate() {
    	let travis = Travis {
            language: "rust".to_string(),
    		rust: vec!["stable".to_string(), "nightly".to_string()],
    		env: vec![
    			"CRATE=boards/feather_m4 EXAMPLES=\"--example=blinky_basic --example=blinky_rtfm\"".to_string(),
    			"CRATE=boards/gemma_m0 FEATURES=\"--features=unproven\"".to_string()
    		],
    		script: vec![
    			"cd $CRATE".to_string(),
    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
    		],
    		matrix: Some(
    			vec![
    				(
    					"allow_failures".to_string(),
    					vec![
    						vec![
    							("rust".to_string(), "nightly".to_string())
    						]
    						.into_iter()
    						.collect()
    					]
    				),
    			]
    			.into_iter()
    			.collect()
    		),
    	};
    	println!("{:#?}", travis);

        let buildkite: Buildkite = super::pipeline_for_travis_config(travis, Some(vec![
            "queue=ecs/agents",
            "task-definition=rust-embedded",
        ]));
    	println!("{:#?}", buildkite);

    	assert_eq!(buildkite, Buildkite {
    		steps: vec![
    			Step {
    				commands: vec![
		    			"cd $CRATE".to_string(),
		    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
		    		],
		    		label: Some(":rust: stable, CRATE=boards/feather_m4 EXAMPLES=\"--example=blinky_basic --example=blinky_rtfm\"".to_string()),
		    		agents: vec![
		    			("queue".to_string(), "ecs/agents".to_string()),
		    			("image".to_string(), "rust:latest".to_string()),
		    			("task-definition".to_string(), "rust-embedded".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		env: vec![
		    			("CRATE".to_string(), "boards/feather_m4".to_string()),
		    			("EXAMPLES".to_string(), "--example=blinky_basic --example=blinky_rtfm".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		soft_fail: Vec::new(),
    			},
    			Step {
    				commands: vec![
		    			"cd $CRATE".to_string(),
		    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
		    		],
		    		label: Some(":rust: stable, CRATE=boards/gemma_m0 FEATURES=\"--features=unproven\"".to_string()),
		    		agents: vec![
		    			("queue".to_string(), "ecs/agents".to_string()),
		    			("image".to_string(), "rust:latest".to_string()),
		    			("task-definition".to_string(), "rust-embedded".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		env: vec![
		    			("CRATE".to_string(), "boards/gemma_m0".to_string()),
		    			("FEATURES".to_string(), "--features=unproven".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		soft_fail: Vec::new(),
    			},
    			Step {
    				commands: vec![
		    			"cd $CRATE".to_string(),
		    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
		    		],
		    		label: Some(":rust: nightly, CRATE=boards/feather_m4 EXAMPLES=\"--example=blinky_basic --example=blinky_rtfm\"".to_string()),
		    		agents: vec![
		    			("queue".to_string(), "ecs/agents".to_string()),
		    			("image".to_string(), "rustlang/rust:nightly".to_string()),
		    			("task-definition".to_string(), "rust-embedded".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		env: vec![
		    			("CRATE".to_string(), "boards/feather_m4".to_string()),
		    			("EXAMPLES".to_string(), "--example=blinky_basic --example=blinky_rtfm".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		soft_fail: vec![
		    			vec![
		    				("exit_status".to_string(), "*".to_string()),
		    			]
		    			.into_iter()
		    			.collect()
		    		],
    			},
    			Step {
    				commands: vec![
		    			"cd $CRATE".to_string(),
		    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
		    		],
		    		label: Some(":rust: nightly, CRATE=boards/gemma_m0 FEATURES=\"--features=unproven\"".to_string()),
		    		agents: vec![
		    			("queue".to_string(), "ecs/agents".to_string()),
		    			("image".to_string(), "rustlang/rust:nightly".to_string()),
		    			("task-definition".to_string(), "rust-embedded".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		env: vec![
		    			("CRATE".to_string(), "boards/gemma_m0".to_string()),
		    			("FEATURES".to_string(), "--features=unproven".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		soft_fail: vec![
		    			vec![
		    				("exit_status".to_string(), "*".to_string()),
		    			]
		    			.into_iter()
		    			.collect()
		    		],
    			}
    		]
    	});
    }
}
