use clap::Parser;
use e6cfg::*;
use schemars::generate::SchemaSettings;

#[derive(Parser)]
struct Cli {
    #[arg(short = 's')]
    gen_schema: bool,
    #[arg(short = 'd')]
    gen_default: bool,
}

fn main() {
    let argv = Cli::parse();

    if argv.gen_schema {
        let settings = SchemaSettings::draft07().for_serialize();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<Cfg>();

        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }

    if argv.gen_default {
        let defaults = serde_json::to_string_pretty(&Cfg::default()).unwrap();

        println!("{}", defaults);
    }
}
