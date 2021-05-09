#[macro_use]
extern crate unhtml_derive;
#[macro_use]
extern crate lazy_static;

use indexmap::IndexMap;
use openapiv3::{Components, OpenAPI, ReferenceOr, SecurityScheme, Tag};
use schema_tweaks::{
    metadata_superclass::metadata_superclass, query_parameters::query_parameters,
    query_superclass::query_superclass, stub_ovf::stub_ovf,
};
use std::io::Read;
mod info;
mod parsers;
mod paths;
mod queries;
mod schema_tweaks;
mod schemas;
use anyhow::{Context, Result};

#[macro_use]
extern crate indexmap;

fn main() -> Result<()> {
    let mut zip_buffer = Vec::new();
    std::io::stdin()
        .read_to_end(&mut zip_buffer)
        .context("Unable to read zip file")?;

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_buffer))
        .context("Unable to parse zip file")?;

    let mut schemas = IndexMap::new();
    query_parameters(
        &mut schemas,
        &queries::queries(&mut zip).context("unable to collect queries")?,
    );
    let content_type_mapping =
        schemas::schemas(&mut schemas, &mut zip).context("Unable to make content type mappings")?;
    stub_ovf(&mut schemas);
    metadata_superclass(&mut schemas);
    query_superclass(&mut schemas);

    let about_info = crate::parsers::about::parse(&{
        let mut html = String::new();
        zip.by_name("about.html")?
            .read_to_string(&mut html)
            .context("Unable to read about info file")?;
        html
    })?;

    let info = info::info(&mut zip, about_info.prodname).context("Unable to parse about info")?;
    let api_version = info
        .version
        .split_ascii_whitespace()
        .rev()
        .next()
        .context("Couldn't determine version")?
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
        paths: paths::paths(&mut zip, content_type_mapping, api_version)
            .context("Unable to collect paths")?,
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
    serde_json::to_writer_pretty(std::io::stdout(), &spec).context("Unable to write JSON")?;
    println!();
    Ok(())
}
