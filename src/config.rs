use miette::{IntoDiagnostic, Result};
use std::{fs, ops::Deref};

#[derive(knus::Decode, Debug)]
pub(crate) struct Category {
    #[knus(argument)]
    pub name: String,
    #[knus(property, default)]
    disabled: bool,
    #[knus(child, unwrap(arguments))]
    pub params: Vec<String>,
}

pub(crate) struct Config(Vec<Category>);

impl Config {
    /// Read filter file
    pub fn from_file(path: &str) -> Result<Self> {
        let filters = fs::read_to_string(path)
            .into_diagnostic()
            .map_err(|err| err.context(format!("Could not read file `{path}`")))?;

        let filters = knus::parse::<Vec<Category>>("config.kdl", &filters)?
            .into_iter()
            .filter(|v| !v.disabled)
            .collect::<Vec<Category>>();

        Ok(Config(filters))
    }

    pub fn flat(&self) -> Vec<String> {
        self.iter().flat_map(|v| v.params.clone()).collect()
    }
}

impl Deref for Config {
    type Target = Vec<Category>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
