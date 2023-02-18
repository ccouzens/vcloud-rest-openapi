use anyhow::Result;
use indexmap::IndexMap;
use openapiv3::{ReferenceOr, Schema};
use std::collections::{BTreeMap};
use std::convert::TryFrom;
use std::io::{Read, Seek};

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

    let mut all_types =
    type_file_names.iter().filter_map(|type_file_name| {
        zip.by_name(&type_file_name).ok()
            .and_then(|mut f| {
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)
                .ok()
                .and_then(|_| xmltree::Element::parse(&buffer as &[u8]).ok()).map(|xml| xmltree::XMLNode::Element(xml))
                .map(|xml| (
                    match xml {
                        xmltree::XMLNode::Element(xmltree::Element { ref attributes, .. })
                            if attributes.contains_key("targetNamespace") =>
                        {
                            attributes.get("targetNamespace").map(|t| match t.as_str() {
                                "http://schemas.dmtf.org/ovf/envelope/1" => "ovf",
                                "http://schemas.dmtf.org/ovf/environment/1" => "ovfenv",
                                "http://schemas.dmtf.org/wbem/wscim/1/cim-schema/2/CIM_ResourceAllocationSettingData" => "rasd",
                                "http://schemas.dmtf.org/wbem/wscim/1/cim-schema/2/CIM_VirtualSystemSettingData" => "vssd",
                                "http://schemas.dmtf.org/wbem/wscim/1/common" => "cim",
                                "http://www.vmware.com/vcloud/meta" => "meta",
                                "http://www.vmware.com/schema/ovf" => "vmw",
                                "http://www.vmware.com/vcloud/extension/v1.5" => "vcloud-ext",
                                "http://www.vmware.com/vcloud/v1.5" => "vcloud",
                                "http://www.vmware.com/vcloud/versions" => "versioning",
                                _ => "vcloud",
                            })
                        }
                        _ => None,
                    },
                    xml,
                ))
            })
    })
    .collect::<Vec<_>>();

    all_types.sort_by_key(|(ns, _)| ns.map_or("", |ns| ns));

    for (ns, type_xml) in all_types.to_owned() {
        let xsd_schema =
            crate::parsers::doc::etc::schema::Schema::try_from((ns, &type_xml, &all_types))?;
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
