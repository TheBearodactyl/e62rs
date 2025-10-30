use {
    crate::{config::options::E62Rs, serve::api::run_api},
    clap::Parser,
    color_eyre::eyre::Result,
    schemars::generate::SchemaSettings,
};

#[derive(Parser)]
pub(crate) struct Cli {
    /// Generate a JSON schemafile based on the defaults
    #[arg(short = 's', long)]
    gen_schema: bool,

    /// Generate the default config file
    #[arg(short = 'd', long)]
    gen_default: bool,

    /// Run as an API
    #[arg(short = 'a', long)]
    api: bool,
}

pub async fn cli() -> Result<()> {
    let argv = Cli::parse();

    if argv.api {
        run_api().await?;
    }

    if argv.gen_schema {
        let settings = SchemaSettings::draft2020_12().for_serialize();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<E62Rs>();

        println!("{}", serde_json::to_string_pretty(&schema)?);
    }

    if argv.gen_default {
        let defaults = &E62Rs::default();
        let defaults_str = toml::to_string_pretty(defaults)?;

        println!("{}", defaults_str);
    }

    Ok(())
}
