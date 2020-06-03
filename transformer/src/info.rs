use openapiv3::Info;
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
    let mut about_file = zip.by_name("about.html")?;

    let mut html = String::new();
    about_file.read_to_string(&mut html)?;

    let about_info = AboutInfo::from_html(&html)?;

    Ok(Info {
        title: about_info.prodname,
        version: about_info.version,
        ..Default::default()
    })
}
