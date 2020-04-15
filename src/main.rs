extern crate clap;

#[macro_use]
extern crate itertools;

#[cfg(test)]
extern crate pretty_assertions;

extern crate shellwords;

extern crate serde;
extern crate serde_yaml;

use std::{
	fs::File,
	io::Read,
	result::Result,
	error::Error,
};

mod travis;
use travis::Travis;

mod buildkite;

fn main() -> Result<(), Box<dyn Error>> {
	let matches = clap::App::new("travis-pipeline")
      .author("Keith Duncan <keith_duncan@me.com>")
      .about("Travis to Buildkite translation layer")
      .arg(clap::Arg::with_name("AGENT_QUERY_RULES")
           .long("agent-query-rules")
           .help("The agent query rules to use for the generated Buildkite steps.")
           .takes_value(true)
           .multiple(true))
      .arg(clap::Arg::with_name("INPUT")
           .help("The path to the travis file to translate")
           .required(true)
           .value_name("FILE")
           .index(1))
      .get_matches();

    let agent_query_rules = matches
    	.values_of("AGENT_QUERY_RULES")
    	.map(clap::Values::collect);

    let file_path = matches.value_of("INPUT").expect("INPUT is required");

	let mut file = File::open(file_path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let travis: Travis = serde_yaml::from_str(&contents)?;
    let buildkite = buildkite::pipeline_for_travis_config(travis, agent_query_rules);

    println!("{}", serde_yaml::to_string(&buildkite)?);

    Ok(())
}

#[cfg(test)]
mod tests {
	use super::{
		Travis,
		Buildkite,
		Step,
		Map,
		pretty_assertions::assert_eq,
	};

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

        let buildkite: Buildkite = super::buildkite_pipeline_for_travis_config(travis, Some(vec![
            "queue=ecs/agents",
            "rust:embedded=true",
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
		    			("rust".to_string(), "stable".to_string()),
		    			("rust:embedded".to_string(), "true".to_string()),
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
		    			("rust".to_string(), "stable".to_string()),
		    			("rust:embedded".to_string(), "true".to_string()),
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
		    			("rust".to_string(), "nightly".to_string()),
		    			("rust:embedded".to_string(), "true".to_string()),
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
		    			("rust".to_string(), "nightly".to_string()),
		    			("rust:embedded".to_string(), "true".to_string()),
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
