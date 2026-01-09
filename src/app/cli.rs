//! cli stuff
use {
    crate::config::options::E62Rs,
    clap::Parser,
    color_eyre::{Report, eyre::Result},
    schemars::generate::SchemaSettings,
    std::{
        fs::OpenOptions,
        io::{BufWriter, Write},
    },
};

/// the CLI
#[derive(Parser)]
pub struct Cli {
    /// Save instead of printing
    #[arg(long)]
    pub save: bool,

    /// Generate a JSON schemafile based on the defaults
    #[arg(short = 's', long)]
    pub gen_schema: bool,

    /// Generate the default config file
    #[arg(short = 'd', long)]
    pub gen_default: bool,

    /// Generate both the schema and the default config file
    #[arg(short = 'a', long)]
    pub gen_all: bool,

    /// Display localization progress
    #[arg(short, long = "localization")]
    pub loc_prog: bool,
}

impl Cli {
    /// run the CLI
    ///
    /// # Errors
    ///
    /// returns an error if it fails to generate and/or save the json schema  
    /// returns an error if it fails to generate and/or save the default config  
    pub async fn run() -> Result<()> {
        let argv = Self::parse();

        if argv.gen_schema || argv.gen_all {
            Self::gen_schema(argv.save)?;
        }

        if argv.gen_default || argv.gen_all {
            Self::gen_defaults(argv.save)?;
        }

        if argv.loc_prog {
            crate::ui::menus::calculate_localization_progress();
        }

        if argv.gen_default || argv.gen_all || argv.gen_schema || argv.save || argv.loc_prog {
            std::process::exit(0);
        }

        Ok(())
    }

    /// save a string to a file
    ///
    /// # Arguments
    ///
    /// * `path` - the path to the file being written
    /// * `contents` - the data to write to the file
    ///
    /// # Errors
    ///
    /// returns an error if it fails to open `path`
    pub fn write_to_file(path: &str, contents: &str) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;
        let mut w = BufWriter::new(file);
        w.write_all(contents.as_bytes()).map_err(Report::new)
    }

    /// generate/save the config schema
    ///
    /// # Arguments
    ///
    /// * `save` - save instead of printing
    ///
    /// # Errors
    ///
    /// returns an error if it fails to convert the schema to a JSON string  
    /// returns an error if it fails to save the schema to `resources/e62rs.schema.json`
    pub fn gen_schema(save: bool) -> Result<()> {
        let settings = SchemaSettings::draft2020_12().for_serialize();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<E62Rs>();
        let schema_str = serde_json::to_string_pretty(&schema)?;

        if save {
            Self::write_to_file("resources/e62rs.schema.json", &schema_str)?;
        } else {
            println!("{}", schema_str);
        }

        Ok(())
    }

    /// generate/save the default config file
    ///
    /// # Arguments
    ///
    /// * `save` - save instead of printing
    ///
    /// # Errors
    ///
    /// returns an error if it fails to convert the default config to TOML  
    /// returns an error if it fails to save the default config to `resources/e62rs.default.toml
    pub fn gen_defaults(save: bool) -> Result<()> {
        let defaults = toml::to_string_pretty(&E62Rs::default())?;

        if save {
            Self::write_to_file("resources/e62rs.default.toml", &defaults)?;
        } else {
            println!("{}", defaults);
        }

        Ok(())
    }
}
