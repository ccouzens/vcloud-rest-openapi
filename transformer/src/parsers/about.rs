use unhtml::FromHtml;

#[derive(FromHtml, Debug)]
pub struct AboutInfo {
    #[html(selector = "meta[name='prodname']", attr = "content")]
    pub prodname: String,
    #[html(selector = "meta[name='version']", attr = "content")]
    pub version: String,
}

pub fn parse(html: &str) -> Result<AboutInfo, unhtml::Error> {
    AboutInfo::from_html(html)
}
