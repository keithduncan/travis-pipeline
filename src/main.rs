#[macro_use]
extern crate itertools;

#[cfg(test)]
extern crate pretty_assertions;

extern crate shellwords;

extern crate serde;
extern crate serde_yaml;

use serde::{Serialize, Deserialize};

use std::{
	env,
	fs::File,
	io::Read,
	result::Result,
	error::Error,
	collections::HashMap as Map,
	convert::From,
};

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct Travis {
	rust: Vec<String>,
	env: Vec<String>,
	script: Vec<String>,
	matrix: Option<Map<String, Vec<Map<String, String>>>>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct Buildkite {
	steps: Vec<Step>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct Step {
	commands: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	label: Option<String>,
	#[serde(skip_serializing_if = "Map::is_empty")]
	agents: Map<String, String>,
	#[serde(skip_serializing_if = "Map::is_empty")]
	env: Map<String, String>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	soft_fail: Vec<Map<String, i32>>,
}

fn buildkite_env_for_travis_env(travis: &str) -> Map<String, String> {
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
						("rust:embedded".to_string(), "true".to_string()),
					]
					.into_iter()
					.collect(),
				env: buildkite_env_for_travis_env(&env),
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
								("exit_status".to_string(), 1)
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

fn main() -> Result<(), Box<Error>> {
	for argument in env::args().into_iter().skip(1) {
		let mut file = File::open(argument)?;
	    let mut contents = String::new();
	    file.read_to_string(&mut contents)?;

	    let travis: Travis = serde_yaml::from_str(&contents)?;
	    let buildkite: Buildkite = travis.into();

	    println!("{}", serde_yaml::to_string(&buildkite)?);
	}

    Ok(())
}

#[cfg(test)]
mod tests {
	use super::{
		Travis,
		Buildkite,
		Step,
		Map,
		pretty_assertions::{
			assert_eq,
			assert_ne
		},
	};

    #[test]
    fn it_works() {
    	assert_eq!(2 + 2, 4);
    }

    #[test]
    fn convert_env() {
    	let travis_env = "CRATE=boards/feather_m4 EXAMPLES=\"--example=blinky_basic --example=blinky_rtfm\"";
    	println!("{:#?}", travis_env);
    	
    	let buildkite_env = super::buildkite_env_for_travis_env(travis_env);
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
    		rust: vec!["stable".to_string(), "nightly".to_string()],
    		env: vec![
    			"CRATE=boards/feather_m4".to_string(),
    			"CRATE=boards/gemma_m0".to_string()
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

    	let buildkite: Buildkite = travis.into();
    	println!("{:#?}", buildkite);

    	assert_eq!(buildkite, Buildkite {
    		steps: vec![
    			Step {
    				commands: vec![
		    			"cd $CRATE".to_string(),
		    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
		    		],
		    		label: Some(":rust: stable, CRATE=boards/feather_m4".to_string()),
		    		agents: vec![
		    			("rust".to_string(), "stable".to_string()),
		    			("rust:embedded".to_string(), "true".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		env: vec![
		    			("CRATE".to_string(), "boards/feather_m4".to_string()),
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
		    		label: Some(":rust: stable, CRATE=boards/gemma_m0".to_string()),
		    		agents: vec![
		    			("rust".to_string(), "stable".to_string()),
		    			("rust:embedded".to_string(), "true".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		env: vec![
		    			("CRATE".to_string(), "boards/gemma_m0".to_string()),
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
		    		label: Some(":rust: nightly, CRATE=boards/feather_m4".to_string()),
		    		agents: vec![
		    			("rust".to_string(), "nightly".to_string()),
		    			("rust:embedded".to_string(), "true".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		env: vec![
		    			("CRATE".to_string(), "boards/feather_m4".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		soft_fail: vec![
		    			vec![
		    				("exit_status".to_string(), 1)
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
		    		label: Some(":rust: nightly, CRATE=boards/gemma_m0".to_string()),
		    		agents: vec![
		    			("rust".to_string(), "nightly".to_string()),
		    			("rust:embedded".to_string(), "true".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		env: vec![
		    			("CRATE".to_string(), "boards/gemma_m0".to_string()),
		    		]
		    		.into_iter()
		    		.collect::<Map<_, _>>(),
		    		soft_fail: vec![
		    			vec![
		    				("exit_status".to_string(), 1)
		    			]
		    			.into_iter()
		    			.collect()
		    		],
    			}
    		]
    	});
    }
}
