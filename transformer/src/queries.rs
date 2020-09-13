use crate::parsers::doc::query::Query;
use std::{
    convert::TryFrom,
    io::{Read, Seek},
};
use zip::read::ZipArchive;

pub fn queries<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
) -> Result<Vec<Query>, Box<dyn std::error::Error>> {
    let mut path_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/queries/"))
        .filter(|n| n.ends_with(".html"))
        .map(|n| n.into())
        .collect::<Vec<String>>();

    path_file_names.sort();

    path_file_names
        .iter()
        .map(|file_name| -> Result<_, Box<dyn std::error::Error>> {
            let mut html = String::new();
            zip.by_name(&file_name)?.read_to_string(&mut html)?;

            Ok(Query::try_from(html.as_str())?)
        })
        .collect()
}
