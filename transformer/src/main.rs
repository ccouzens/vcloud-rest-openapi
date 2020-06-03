#[macro_use]
extern crate unhtml_derive;

use openapiv3::OpenAPI;
use serde::Serialize;
use std::io::Read;
use unhtml::FromHtml;
mod info;

#[derive(FromHtml, Debug, Serialize)]
#[html]
struct RawOperation {
    #[html(selector = "td:first-child > a", attr = "href")]
    href: String,
    #[html(selector = "td:first-child", attr = "inner")]
    route: String,
    #[html(selector = "td:nth-child(2)", attr = "inner")]
    description: String,
}

#[derive(FromHtml, Debug, Serialize)]
struct LandingOperations {
    #[html(selector = "h2", attr = "inner")]
    subtitle: String,
    #[html(selector = "tr + tr")]
    raws: Vec<RawOperation>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut zip_buffer = Vec::new();
    std::io::stdin().read_to_end(&mut zip_buffer)?;

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_buffer))?;
    let mut admin_op_file = zip.by_name("doc/landing-admin_operations.html")?;

    let mut html = String::new();
    admin_op_file.read_to_string(&mut html)?;
    std::mem::drop(admin_op_file);
    // let operations = LandingOperations::from_html(&html)?;

    let spec = OpenAPI {
        openapi: "3.0.2".into(),
        info: info::info(&mut zip)?,
        ..Default::default()
    };
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
