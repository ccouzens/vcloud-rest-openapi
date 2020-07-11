use crate::parsers::doc::operation::{Method, Operation};
use openapiv3::Paths;
use std::{
    convert::TryFrom,
    io::{Read, Seek},
};
use zip::read::ZipArchive;

pub fn paths<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Paths, Box<dyn std::error::Error>> {
    let mut path_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/operations/"))
        .filter(|n| n.ends_with(".html"))
        .map(|n| n.into())
        .collect::<Vec<String>>();

    path_file_names.sort();

    let mut paths = Paths::new();

    for file_name in path_file_names {
        let mut html = String::new();
        zip.by_name(&file_name)?.read_to_string(&mut html)?;

        let operation = Operation::try_from(html.as_str())?;
        if let openapiv3::ReferenceOr::Item(path_item) = paths
            .entry(operation.path.clone())
            .or_insert(openapiv3::ReferenceOr::Item(openapiv3::PathItem::default()))
        {
            match operation.method {
                Method::Get => path_item.get = Some(operation.into()),
                Method::Post => path_item.post = Some(operation.into()),
                Method::Put => path_item.put = Some(operation.into()),
                Method::Delete => path_item.delete = Some(operation.into()),
            }
        };
    }
    Ok(paths)
}
