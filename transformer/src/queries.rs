use crate::parsers::doc::query::Query;
use anyhow::{Context, Result};
use std::{
    convert::TryFrom,
    io::{Read, Seek},
};
use zip::read::ZipArchive;

pub fn queries<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Vec<Query>> {
    let mut path_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/queries/"))
        .filter(|n| n.ends_with(".html"))
        .map(|n| n.into())
        .collect::<Vec<String>>();

    path_file_names.sort();

    path_file_names
        .iter()
        .map(|file_name| -> Result<_> {
            let mut html = String::new();
            zip.by_name(file_name)
                .with_context(|| format!("Unable to find file {} in zip", file_name))?
                .read_to_string(&mut html)
                .with_context(|| format!("Unable to read file {} from zip", file_name))?;

            Query::try_from(html.as_str())
                .with_context(|| format!("Unable to parse file {} into query", file_name))
        })
        .collect()
}
