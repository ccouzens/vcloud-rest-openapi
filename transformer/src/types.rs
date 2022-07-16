use crate::parsers::doc::r#type::Type;
use anyhow::{Context, Result};

use std::{
    collections::BTreeMap,
    convert::TryFrom,
    io::{Read, Seek},
};
use zip::read::ZipArchive;

pub fn types<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<BTreeMap<String, Type>> {
    let mut path_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/types/"))
        .filter(|n| n.ends_with(".html"))
        .map(|n| n.into())
        .collect::<Vec<String>>();

    path_file_names.sort();

    let types = path_file_names
        .iter()
        .flat_map(|file_name| -> Result<_> {
            let mut html = String::new();
            zip.by_name(file_name)
                .with_context(|| format!("Unable to find file {} in zip", file_name))?
                .read_to_string(&mut html)
                .with_context(|| format!("Unable to read file {} from zip", file_name))?;

            Type::try_from(html.as_str())
                .with_context(|| format!("Unable to parse file {} into type", file_name))
        })
        .map(|t| (t.name.to_string(), t))
        .collect();
    Ok(types)
}
