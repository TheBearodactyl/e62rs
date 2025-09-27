use e6cfg::*;
use schemars::generate::SchemaSettings;

fn main() {
    let settings = SchemaSettings::draft07().for_serialize();
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<Cfg>();

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
