#[macro_use]
extern crate itertools;

extern crate shellwords;

extern crate serde;
extern crate serde_yaml;

use serde::{Serialize, Deserialize};

use std::{
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
	agents: Map<String, String>,
	env: Map<String, String>,
	soft_fail: Option<Vec<Map<String, i32>>>,
}

impl From<Travis> for Buildkite {
	fn from(travis: Travis) -> Buildkite {
		let mut steps: Vec<Step> = Vec::new();

		for (rust, env) in iproduct!(travis.rust, travis.env) {
			let buildkite_env = shellwords::split(&env)
				.expect("env parses")
				.into_iter()
				.map(|string| {
					let key_value: Vec<_> = string.split('=').collect();
					(key_value[0].to_string(), key_value[1].to_string())
				})
				.collect();

			steps.push(Step {
				commands: travis.script.clone(),
				agents: vec![
						("rust".to_string(), rust.clone()),
						("rust:embedded".to_string(), "true".to_string()),
					]
					.into_iter()
					.collect(),
				env: buildkite_env,
				soft_fail: None,
			});
		}

		Buildkite {
			steps: steps,
		}
	}
}

fn main() -> Result<(), Box<Error>> {
	let mut file = File::open("example/travis.yml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Travis = serde_yaml::from_str(&contents)?;

    println!("{:#?}", config);

    Ok(())
}

#[cfg(test)]
mod tests {
	use super::{
		Travis,
		Buildkite,
		Step,
		Map
	};

    #[test]
    fn it_works() {
    	assert_eq!(2 + 2, 4);
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
    		matrix: None,
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
		    		soft_fail: None,
    			},
    			Step {
    				commands: vec![
		    			"cd $CRATE".to_string(),
		    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
		    		],
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
		    		soft_fail: None,
    			},
    			Step {
    				commands: vec![
		    			"cd $CRATE".to_string(),
		    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
		    		],
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
		    		soft_fail: None,
    			},
    			Step {
    				commands: vec![
		    			"cd $CRATE".to_string(),
		    			"cargo build ${EXAMPLES:---examples} $FEATURES".to_string()
		    		],
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
		    		soft_fail: None,
    			}
    		]
    	});
    }
}
