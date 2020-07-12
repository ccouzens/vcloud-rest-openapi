#[macro_use]
extern crate unhtml_derive;

use openapiv3::{Components, OpenAPI};
use std::io::Read;
mod info;
mod parsers;
mod paths;
mod schemas;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut zip_buffer = Vec::new();
    std::io::stdin().read_to_end(&mut zip_buffer)?;

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_buffer))?;

    let (schemas, content_type_mapping) = schemas::schemas(&mut zip)?;
    let spec = OpenAPI {
        openapi: "3.0.2".into(),
        info: info::info(&mut zip)?,
        components: Some(Components {
            schemas,
            ..Default::default()
        }),
        paths: paths::paths(&mut zip, content_type_mapping)?,
        ..Default::default()
    };
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
