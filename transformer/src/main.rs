#[macro_use]
extern crate unhtml_derive;
#[macro_use]
extern crate lazy_static;

use openapiv3::{Components, OpenAPI, ReferenceOr, SecurityScheme, Tag};
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

    let about_info = crate::parsers::about::parse(&{
        let mut html = String::new();
        zip.by_name("about.html")?.read_to_string(&mut html)?;
        html
    })?;

    let info = info::info(&mut zip, about_info.prodname)?;
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
        tags: vec![
            Tag {
                name: "user".into(),
                description: Some(html2md::parse_html(&about_info.user_tag)),
                external_docs: None,
            },
            Tag {
                name: "admin".into(),
                description: Some(html2md::parse_html(&about_info.admin_tag)),
                external_docs: None,
            },
            Tag {
                name: "extension".into(),
                description: Some(html2md::parse_html(&about_info.extension_tag)),
                external_docs: None,
            },
        ],
        ..Default::default()
    };
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
