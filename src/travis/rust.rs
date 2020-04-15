use core::str::FromStr;

pub enum Rust {
    Stable,
    Beta,
    Nightly,
    // SemVer release number
    Release(String),
}

impl Rust {
    // Get a docker image for this travis rust tag
    pub fn image(&self) -> Option<String> {
    	Some(match self {
    		&Rust::Stable => "rust:latest".to_string(),
    		&Rust::Nightly => "rustlang/rust:nightly".to_string(),
    		&Rust::Release(ref ver) => format!("rust:{}", ver),

    		// There are no official images with rust:beta installed, womp
    		&Rust::Beta => return None,
    	})
    }
}

impl FromStr for Rust {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"stable" => Rust::Stable,
			"beta" => Rust::Beta,
			"nightly" => Rust::Nightly,
			v => Rust::Release(v.to_string()),
		})
	}
}