use unhtml::FromHtml;

#[derive(FromHtml, Debug)]
pub struct AboutInfo {
    #[html(selector = "meta[name='prodname']", attr = "content")]
    pub prodname: String,
    #[html(selector = "meta[name='version']", attr = "content")]
    pub version: String,
    #[html(
        selector = "div.section > table.DefinitionList > tbody > tr > td.dddef",
        attr = "inner"
    )]
    pub user_tag: String,
    #[html(
        selector = "div.section > table.DefinitionList > tbody > tr:nth-child(2) > td.dddef",
        attr = "inner"
    )]
    pub admin_tag: String,
    #[html(
        selector = "div.section > table.DefinitionList > tbody > tr:nth-child(3) > td.dddef",
        attr = "inner"
    )]
    pub extension_tag: String,
}

pub fn parse(html: &str) -> Result<AboutInfo, unhtml::Error> {
    AboutInfo::from_html(html)
}
