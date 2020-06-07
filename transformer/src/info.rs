use javascript_lexer::Lexer;
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
    let mut html = String::new();
    zip.by_name("about.html")?.read_to_string(&mut html)?;

    let about_info = AboutInfo::from_html(&html)?;

    let mut javascript = String::new();
    zip.by_name("doc/commonRes.js")?
        .read_to_string(&mut javascript)?;
    dbg!(Lexer::lex_tokens(&javascript)?
        .iter()
        .filter_map(|t| match t {
            javascript_lexer::token::Token::StringLiteral(s) => Some(s),
            _ => None,
        })
        .collect::<Vec<_>>());

    Ok(Info {
        title: about_info.prodname,
        version: about_info.version,
        ..Default::default()
    })
}
