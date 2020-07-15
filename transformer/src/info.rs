use openapiv3::{Info, License};
use std::io::{Read, Seek};
use zip::read::ZipArchive;

pub fn info<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    title: String,
) -> Result<Info, Box<dyn std::error::Error>> {
    let common_res = crate::parsers::doc::common_res::parse(&{
        let mut javascript = Vec::new();
        zip.by_name("doc/commonRes.js")?
            .read_to_end(&mut javascript)?;
        javascript
    })?;
    Ok(Info {
        title,
        version: common_res.version_information,
        license: Some(License {
            name: common_res.copyright,
            ..Default::default()
        }),
        ..Default::default()
    })
}
