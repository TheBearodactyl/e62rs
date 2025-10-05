use clap::Parser;
use e6cfg::*;
use schemars::generate::SchemaSettings;

#[derive(Parser)]
struct Cli {
    /// Generate the JSON schema based on the defaults
    #[arg(short = 's')]
    gen_schema: bool,

    /// Generate the default TOML file
    #[arg(short = 'd')]
    gen_default: bool,
}

fn main() {
    let argv = Cli::parse();

    if argv.gen_schema {
        let settings = SchemaSettings::draft2020_12().for_serialize();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<E62Rs>();

        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }

    if argv.gen_default {
        let defaults = &E62Rs::default();
        let defaults_str = toml::to_string_pretty(defaults).expect("Failed to serialize");

        println!("{}", defaults_str);
    }
}
