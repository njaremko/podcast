use anyhow::Result;

const PACKAGE_TOML: &str = include_str!("../Cargo.toml");

#[derive(Deserialize)]
struct Config {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    version: String,
}

pub fn get_cli_version() -> Result<String> {
    let parsed_cli_toml: Config = toml::from_str(PACKAGE_TOML)?;
    return Ok(parsed_cli_toml.package.version);
}
