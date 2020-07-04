use crate::parsers::doc::etc::object_type::ObjectType;
use crate::parsers::doc::etc::primitive_type::ParsePrimitiveTypeError;
use crate::parsers::doc::etc::simple_type::SimpleType;
#[cfg(test)]
use serde_json::json;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub(super) enum Type {
    ObjectType(ObjectType),
    SimpleType(SimpleType),
}

#[derive(Error, Debug, PartialEq)]
pub enum TypeParseError {
    #[error("not a complex or simple type node")]
    NotTypeNode,
    #[error("missing name attribute")]
    MissingName,
    #[error("failure to parse PrimitiveType")]
    PrimitiveTypeParseError(#[from] ParsePrimitiveTypeError),
    #[error("Missing base attribute")]
    MissingBase,
    #[error("Missing item type attribute")]
    MissingItemTypeValue,
}

impl TryFrom<&xmltree::XMLNode> for Type {
    type Error = TypeParseError;
    fn try_from(value: &xmltree::XMLNode) -> Result<Self, Self::Error> {
        match ObjectType::try_from(value) {
            Err(TypeParseError::NotTypeNode) => Ok(Type::SimpleType(SimpleType::try_from(value)?)),
            Ok(object) => Ok(Type::ObjectType(object)),
            Err(e) => Err(e),
        }
    }
}

impl From<&Type> for openapiv3::Schema {
    fn from(t: &Type) -> Self {
        match t {
            Type::ObjectType(c) => Self::from(c),
            Type::SimpleType(s) => Self::from(s),
        }
    }
}

#[test]
fn parse_type_wth_parent_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:annotation>
            <xs:appinfo>
                <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
            </xs:appinfo>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A simple type to test the parser
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="BaseType">
                <xs:sequence>
                    <xs:element name="OptionalString" type="xs:string" minOccurs="0">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that may or may not be here
                            </xs:documentation>
                            <xs:documentation source="required">false</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                    <xs:element name="RequiredString" type="xs:string" minOccurs="1">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that will be here
                            </xs:documentation>
                            <xs:documentation source="required">true</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                </xs:sequence>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "TestType",
          "description": "A simple type to test the parser",
          "allOf": [
            {
              "$ref": "#/components/schemas/BaseType"
            },
            {
              "type": "object",
              "properties": {
                "optionalString": {
                  "nullable": true,
                  "description": "String that may or may not be here",
                  "type": "string"
                },
                "requiredString": {
                  "description": "String that will be here",
                  "type": "string"
                }
              },
              "required": [
                "optionalString",
                "requiredString"
              ],
              "additionalProperties": false
            }
          ]
        }
        )
    );
}

#[test]
fn parse_type_that_is_attribute_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:annotation>
            <xs:appinfo>
                <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
            </xs:appinfo>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A simple type to test the parser
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="BaseType">
                <xs:attribute name="requiredAttribute" type="xs:string" use="required">
                    <xs:annotation>
                        <xs:documentation source="modifiable">none</xs:documentation>
                        <xs:documentation>
                            A field that comes from an attribute.
                        </xs:documentation>
                        <xs:documentation source="required">true</xs:documentation>
                    </xs:annotation>
                </xs:attribute>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "TestType",
          "description": "A simple type to test the parser",
          "allOf": [
            {
              "$ref": "#/components/schemas/BaseType"
            },
            {
              "type": "object",
              "properties": {
                "requiredAttribute": {
                  "readOnly": true,
                  "description": "A field that comes from an attribute.",
                  "type": "string"
                }
              },
              "required": [
                "requiredAttribute"
              ],
              "additionalProperties": false
            }
          ]
        })
    );
}

#[test]
fn parse_type_that_is_attribute_but_not_extension_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:annotation>
            <xs:appinfo>
                <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
            </xs:appinfo>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A simple type to test the parser
            </xs:documentation>
        </xs:annotation>
        <xs:attribute name="requiredAttribute" type="xs:string" use="required">
            <xs:annotation>
                <xs:documentation source="modifiable">none</xs:documentation>
                <xs:documentation>
                    A field that comes from an attribute.
                </xs:documentation>
                <xs:documentation source="required">true</xs:documentation>
            </xs:annotation>
        </xs:attribute>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "TestType",
          "description": "A simple type to test the parser",
          "type": "object",
          "properties": {
            "requiredAttribute": {
              "readOnly": true,
              "description": "A field that comes from an attribute.",
              "type": "string"
            }
          },
          "required": [
            "requiredAttribute"
          ],
          "additionalProperties": false
        }
        )
    );
}

#[test]
fn simple_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="CoinType" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:annotation>
            <xs:appinfo><meta:version added-in="5.6"/></xs:appinfo>
            <xs:documentation source="since">5.6</xs:documentation>
            <xs:documentation xml:lang="en">
                An enumeration of the sides of a coin
            </xs:documentation>
        </xs:annotation>
        <xs:restriction base="xs:string">
            <xs:enumeration value="Heads"/>
            <xs:enumeration value="Tails"/>
        </xs:restriction>
    </xs:simpleType>
    "#;

    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "An enumeration of the sides of a coin",
            "type": "string",
            "enum": ["Heads", "Tails"],
            "title": "CoinType"
        })
    );
}

#[test]
fn simple_type_with_pattern_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="HttpsType"xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:restriction base="xs:anyURI">
            <xs:pattern value="https://.+"/>
        </xs:restriction>
    </xs:simpleType>
    "#;

    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "type": "string",
            "format": "uri",
            "pattern": "https://.+",
            "title": "HttpsType"
        })
    );
}

#[test]
fn simple_type_with_min_inclusive_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="DaysInMonth" xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:restriction base="xs:int">
            <xs:minInclusive value="28"/>
        </xs:restriction>
    </xs:simpleType>
    "#;

    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "type": "integer",
            "format": "int32",
            "minimum": 28,
            "title": "DaysInMonth"
        })
    );
}

#[test]
fn simple_type_with_list_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="FavouriteFoods" xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:list itemType="xs:string"/>
    </xs:simpleType>
    "#;

    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "type": "array",
            "items": { "type": "string" },
            "title": "FavouriteFoods"
        })
    );
}

#[test]
fn simplest_simple_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType xmlns:xs="http://www.w3.org/2001/XMLSchema">
        <xs:restriction base="xs:string">
        </xs:restriction>
    </xs:simpleType>
    "#;

    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({"type": "string"})
    );
}

#[test]
fn base_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" abstract="true" name="BaseType">
        <xs:annotation>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A base abstract type for all the types.
            </xs:documentation>
        </xs:annotation>

        <xs:sequence>
            <xs:element name="BaseField" type="xs:string" minOccurs="0">
                <xs:annotation>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        A base field for the base type
                    </xs:documentation>
                    <xs:documentation source="required">false</xs:documentation>
                </xs:annotation>
            </xs:element>
        </xs:sequence>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "additionalProperties": false,
            "description": "A base abstract type for all the types.",
            "properties": {
                "baseField": {
                    "description": "A base field for the base type",
                    "nullable": true,
                    "type": "string"
                }
            },
            "required": ["baseField"],
            "title": "BaseType",
            "type": "object"
        })
    );
}

#[test]
fn parent_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:annotation>
            <xs:appinfo>
                <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
            </xs:appinfo>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A simple type to test the parser
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="BaseType">
                <xs:sequence>
                    <xs:element name="OptionalString" type="xs:string" minOccurs="0">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that may or may not be here
                            </xs:documentation>
                            <xs:documentation source="required">false</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                    <xs:element name="RequiredString" type="xs:string" minOccurs="1">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that will be here
                            </xs:documentation>
                            <xs:documentation source="required">true</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                </xs:sequence>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A simple type to test the parser",
            "allOf": [
                {
                    "$ref": "#/components/schemas/BaseType"
                },
                {
                  "additionalProperties": false,
                  "properties": {
                    "optionalString": {
                        "description": "String that may or may not be here",
                        "nullable": true,
                        "type": "string"
                    },
                    "requiredString": {
                        "description": "String that will be here",
                        "type": "string"
                    }
                  },
                  "required": ["optionalString", "requiredString"],
                  "type": "object"
              }
            ],
            "title": "TestType",
        })
    );
}

#[test]
fn parse_group_test() {
    let xml: &[u8] = br#"
    <xs:group xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="SimpleGroup">
        <xs:sequence>
            <xs:element name="Field1" type="xs:int" minOccurs="1" maxOccurs="1">
                <xs:annotation>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        The first field in the group.
                    </xs:documentation>
                    <xs:documentation source="required">true</xs:documentation>
                </xs:annotation>
            </xs:element>
            <xs:element name="Field2" type="xs:string" minOccurs="0" maxOccurs="1">
                <xs:annotation>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        The second field in the group.
                    </xs:documentation>
                    <xs:documentation source="required">false</xs:documentation>
                </xs:annotation>
            </xs:element>
        </xs:sequence>
    </xs:group>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "title": "SimpleGroup",
          "type": "object",
          "properties": {
            "field1": {
              "description": "The first field in the group.",
              "type": "integer",
              "format": "int32"
            },
            "field2": {
              "nullable": true,
              "description": "The second field in the group.",
              "type": "string"
            }
          },
          "required": [
            "field1",
            "field2"
          ],
          "additionalProperties": false
        })
    );
}
