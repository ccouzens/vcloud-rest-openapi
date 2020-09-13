use indexmap::IndexMap;
use openapiv3::{ReferenceOr, Schema};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::io::{Read, Seek};
use zip::read::ZipArchive;

pub fn schemas<R: Read + Seek>(
    output: &mut IndexMap<String, ReferenceOr<Schema>>,
    zip: &mut ZipArchive<R>,
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let mut type_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/etc/"))
        .filter(|n| n.ends_with(".xsd"))
        .filter(|&n| n != "doc/etc/schemas/external/xml.xsd")
        .filter(|n| !n.starts_with("doc/etc/schemas/external/ovf1.1/"))
        .map(|n| n.into())
        .collect::<Vec<String>>();

    type_file_names.sort();

    let mut content_type_mapping = BTreeMap::new();

    for type_file_name in type_file_names {
        let mut bytes = Vec::new();
        zip.by_name(&type_file_name)?.read_to_end(&mut bytes)?;

        let namespace = if type_file_name.starts_with("doc/etc/1.5/schemas/extension/") {
            "vcloud-ext"
        } else if type_file_name.starts_with("doc/etc/1.5/schemas/") {
            "vcloud"
        } else if type_file_name.starts_with("doc/etc/schemas/versioning/") {
            "versioning"
        } else if type_file_name.starts_with("doc/etc/schemas/external/ovf1.1/") {
            "ovf"
        } else {
            "unknown"
        };

        let xsd_schema =
            crate::parsers::doc::etc::schema::Schema::try_from((&bytes as &[u8], namespace))?;
        output.extend(
            Vec::<Schema>::from(&xsd_schema)
                .into_iter()
                .filter_map(|s| {
                    s.schema_data
                        .title
                        .clone()
                        .map(|title| (title, ReferenceOr::Item(s)))
                }),
        );
        content_type_mapping.extend(xsd_schema.content_types_names());
    }

    Ok(content_type_mapping)
}
