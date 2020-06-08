use unhtml::FromHtml;

#[derive(FromHtml, Debug)]
#[html]
pub struct RawOperation {
    #[html(selector = "td:first-child > a", attr = "href")]
    pub href: String,
    #[html(selector = "td:first-child", attr = "inner")]
    route: String,
    #[html(selector = "td:nth-child(2)", attr = "inner")]
    pub description: String,
}

impl RawOperation {
    pub fn method(&self) -> Result<&str, Box<dyn std::error::Error>> {
        Ok(self
            .route
            .split_ascii_whitespace()
            .nth(0)
            .ok_or("Failed to get method")?)
    }

    pub fn path(&self) -> Result<&str, Box<dyn std::error::Error>> {
        Ok(self
            .route
            .split_ascii_whitespace()
            .nth(1)
            .ok_or("Failed to get path")?)
    }
}

#[derive(FromHtml, Debug)]
pub struct LandingOperations {
    #[html(selector = "h2", attr = "inner")]
    pub subtitle: String,
    #[html(selector = "tr + tr")]
    pub raws: Vec<RawOperation>,
}

pub fn parse(html: &str) -> Result<LandingOperations, unhtml::Error> {
    LandingOperations::from_html(html)
}
