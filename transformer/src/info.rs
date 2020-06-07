use openapiv3::{Info, License};
use std::io::{Read, Seek};
use unhtml::FromHtml;
use zip::read::ZipArchive;

#[derive(FromHtml, Debug)]
struct AboutInfo {
    #[html(selector = "meta[name='prodname']", attr = "content")]
    prodname: String,
    #[html(selector = "meta[name='version']", attr = "content")]
    version: String,
}

pub fn info<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Info, Box<dyn std::error::Error>> {
    let mut html = String::new();
    zip.by_name("about.html")?.read_to_string(&mut html)?;

    let about_info = AboutInfo::from_html(&html)?;

    let mut javascript = Vec::new();
    zip.by_name("doc/commonRes.js")?
        .read_to_end(&mut javascript)?;

    let common_res = crate::parsers::doc::common_res::parse(&javascript);
    Ok(Info {
        title: about_info.prodname,
        version: common_res.version_information,
        license: Some(License {
            name: common_res.copyright,
            ..Default::default()
        }),
        ..Default::default()
    })
}
