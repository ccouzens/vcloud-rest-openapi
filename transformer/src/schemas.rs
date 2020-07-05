use indexmap::IndexMap;
use openapiv3::{ReferenceOr, Schema};
use std::convert::TryFrom;
use std::io::{Read, Seek};
use zip::read::ZipArchive;

pub fn schemas<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
) -> Result<IndexMap<String, ReferenceOr<Schema>>, Box<dyn std::error::Error>> {
    let mut output = IndexMap::new();
    let mut type_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/etc/"))
        .filter(|n| n.ends_with(".xsd"))
        .filter(|&n| n != "doc/etc/schemas/external/xml.xsd")
        .filter(|n| !n.starts_with("doc/etc/schemas/external/ovf1.1/"))
        .map(|n| n.into())
        .collect::<Vec<String>>();

    type_file_names.sort();

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
    }

    output.insert(
        "ovf_Section_Type".to_owned(),
        ReferenceOr::Item(openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(Default::default())),
        }),
    );

    Ok(output)
}
