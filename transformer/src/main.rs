#[macro_use]
extern crate unhtml_derive;

use openapiv3::OpenAPI;
use std::io::Read;
mod info;
mod parsers;
mod paths;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut zip_buffer = Vec::new();
    std::io::stdin().read_to_end(&mut zip_buffer)?;

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_buffer))?;

    let spec = OpenAPI {
        openapi: "3.0.2".into(),
        info: info::info(&mut zip)?,
        paths: paths::paths(&mut zip)?,
        ..Default::default()
    };
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
