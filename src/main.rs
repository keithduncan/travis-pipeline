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
	let matches = clap::App::new(env!("CARGO_PKG_NAME"))
      .version(env!("CARGO_PKG_VERSION"))
      .author(env!("CARGO_PKG_AUTHORS"))
      .about(env!("CARGO_PKG_DESCRIPTION"))
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
