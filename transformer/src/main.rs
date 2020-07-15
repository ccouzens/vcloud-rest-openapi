#[macro_use]
extern crate unhtml_derive;

use openapiv3::{Components, OpenAPI, ReferenceOr, SecurityScheme};
use std::io::Read;
mod info;
mod parsers;
mod paths;
mod schemas;

#[macro_use]
extern crate indexmap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut zip_buffer = Vec::new();
    std::io::stdin().read_to_end(&mut zip_buffer)?;

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_buffer))?;

    let (schemas, content_type_mapping) = schemas::schemas(&mut zip)?;

    let info = info::info(&mut zip)?;
    let api_version = info
        .version
        .split_ascii_whitespace()
        .rev()
        .next()
        .ok_or("Couldn't determine version")?
        .to_string();

    let spec = OpenAPI {
        openapi: "3.0.2".into(),
        info,
        components: Some(Components {
            schemas,
            security_schemes: indexmap! {
                "basicAuth".into() => ReferenceOr::Item(
                    SecurityScheme::HTTP { scheme: "basic".into(), bearer_format: None}),
                "bearerAuth".into() => ReferenceOr::Item(
                    SecurityScheme::HTTP { scheme: "bearer".into(), bearer_format: None})
            },
            ..Default::default()
        }),
        paths: paths::paths(&mut zip, content_type_mapping, api_version)?,
        ..Default::default()
    };
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
