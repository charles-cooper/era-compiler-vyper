//!
//! The `vyper --standard-json` output representation.
//!

pub mod contract;
pub mod error;

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::project::contract::vyper::Contract as VyperContract;
use crate::project::Project;

use self::contract::Contract;
use self::error::Error;

///
/// The `vyper --standard-json` output representation.
///
/// Unlike in the Solidity compiler, it is not passed up to the hardhat plugin, but only used here
/// internally to reduce the number of calls to the `vyper` subprocess.
///
#[derive(Debug, Deserialize)]
pub struct Output {
    /// The file-contract hashmap.
    #[serde(rename = "contracts")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<BTreeMap<String, BTreeMap<String, Contract>>>,
    /// The compilation errors and warnings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<Error>>,
    /// The `vyper` compiler long version.
    #[serde(rename = "compiler")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_version: Option<String>,
}

impl Output {
    ///
    /// Converts the `vyper` JSON output into a convenient project representation.
    ///
    pub fn try_into_project(mut self, version: &semver::Version) -> anyhow::Result<Project> {
        let files = match self.files.take() {
            Some(files) => files,
            None => {
                anyhow::bail!(
                    "{}",
                    self.errors
                        .as_ref()
                        .map(|errors| serde_json::to_string_pretty(errors).expect("Always valid"))
                        .unwrap_or_else(|| "Unknown project assembling error".to_owned())
                );
            }
        };

        let mut project_contracts = BTreeMap::new();
        for (path, file) in files.into_iter() {
            for (name, contract) in file.into_iter() {
                let full_path = format!("{path}:{name}");

                let project_contract =
                    VyperContract::new(contract.metadata, contract.ir, contract.evm.abi);
                project_contracts.insert(full_path, project_contract.into());
            }
        }

        Ok(Project::new(version.to_owned(), project_contracts))
    }
}
