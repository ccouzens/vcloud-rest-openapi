use openapiv3::{PathItem, Paths};
use std::io::{Read, Seek};
use zip::read::ZipArchive;

pub fn paths<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Paths, Box<dyn std::error::Error>> {
    let admin_ops = crate::parsers::doc::landing_gen_operations::parse(&{
        let mut html = String::new();
        zip.by_name("doc/landing-admin_operations.html")?
            .read_to_string(&mut html)?;
        html
    })?;

    let mut paths = Paths::new();
    for raw_op in admin_ops.raws {
        paths.entry(raw_op.path()?.into()).or_insert_with(|| {
            openapiv3::ReferenceOr::Item(openapiv3::PathItem {
                ..Default::default()
            })
        });
    }
    Ok(paths)
}
