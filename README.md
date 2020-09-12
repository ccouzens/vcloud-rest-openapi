# VMware Cloud Director Rest OpenAPI definitions

This is a repository of OpenAPI definitions for vCloud's Rest API. They are
automatically generated from the public documentation to reduce human error,
reduce human effort and to make it easy to stay up-to date.

vCloud Director has an
[official](https://vdc-download.vmware.com/vmwb-repository/dcr-public/772aa4c5-7e61-4d80-8432-b8e0d821c969/2747ec83-6aef-4560-b1d1-55ed9adc4e73/vcd-openapi-docs.html)
[OpenAPI](https://github.com/vmware/vcd-api-schemas/blob/master/schemas/openapi/src/main/resources/schemas/vcloud-openapi-schemas.yaml),
but it contains little of the functionality of the
[Rest API](https://code.vmware.com/apis/912/vmware-cloud-director).

[API definition for VMware Cloud Director 10.1](./34.0.json)

Other versions can be found at the top level directory of this repository.

## Problem Description

VMware Cloud Director Rest API supports responses and requests as both JSON and
XML. But only XML is officially documented by VMWare. JSON is better suited for
OpenAPI and modern usage. The mapping from XML to JSON is predictable.

Types are documented by VMware in XSD (XML Schema Definition). XSD is a
complicated format. It supports lots of concepts and it has multiple ways of
expressing a single concept. There are edge cases where I've not yet
incorporated an XSD concept.

If you find an issue, tell me about it using
[Github](https://github.com/ccouzens/vcloud-rest-openapi/issues) and I shall try
and address it.

## License

The transformer (code that does the converting of the published documentation to
OpenAPI) is MIT Licensed.

I've assigned the copyright of the OpenAPI definitions to VMware as it's a
derivative of their documentation (but does
[copyright apply to APIs](https://en.wikipedia.org/wiki/Google_v._Oracle_America)?)
