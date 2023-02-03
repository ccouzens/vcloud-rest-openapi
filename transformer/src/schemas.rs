use anyhow::{Context, Result};
use indexmap::IndexMap;
use openapiv3::{ReferenceOr, Schema};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::io::{Read, Seek};
use std::iter::FromIterator;
use zip::read::ZipArchive;

pub fn schemas<R: Read + Seek>(
    output: &mut IndexMap<String, ReferenceOr<Schema>>,
    zip: &mut ZipArchive<R>,
) -> Result<BTreeMap<String, String>> {
    let mut type_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/etc/"))
        .filter(|n| n.ends_with(".xsd"))
        .filter(|n| !n.starts_with("doc/etc/etc/snapshot"))
        .filter(|&n| n != "doc/etc/schemas/external/xml.xsd")
        .filter(|&n| n != "doc/etc/etc/schemas/external/xml.xsd")
        .map(|n| n.into())
        .collect::<Vec<String>>();

    type_file_names.sort();

    let mut content_type_mapping = BTreeMap::new();

    let types_files: HashMap<&String, xmltree::XMLNode> =
        HashMap::from_iter(type_file_names.iter().filter_map(|type_file_name| {
            zip.by_name(&type_file_name)
                .map(|mut f| {
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer)
                        .map(|_| {
                            (
                                type_file_name,
                                xmltree::XMLNode::Element(
                                    xmltree::Element::parse(&buffer as &[u8]).unwrap(),
                                ),
                            )
                        })
                        .unwrap()
                })
                .ok()
        }));

    let all_types: Vec<&xmltree::XMLNode> = types_files.values().collect();

    for (type_file_name, ref type_xml) in types_files.to_owned() {
        let xsd_schema = crate::parsers::doc::etc::schema::Schema::try_from((type_xml, &all_types))
            .with_context(|| format!("Unable to parse {} as schema", type_file_name))?;
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
