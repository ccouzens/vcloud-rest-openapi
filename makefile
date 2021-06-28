specs = \
27.0-rest-api.json \
29.0-rest-api.json \
30.0-rest-api.json \
31.0-rest-api.json \
32.0-rest-api.json \
32.0-cloudapi.json \
33.0-rest-api.json \
33.0-cloudapi.json \
34.0-rest-api.json \
34.0-cloudapi.json \
35.0-rest-api.json \
35.2-rest-api.json \
27.0-rest-api.yml \
29.0-rest-api.yml \
30.0-rest-api.yml \
31.0-rest-api.yml \
32.0-rest-api.yml \
32.0-cloudapi.yml \
33.0-rest-api.yml \
33.0-cloudapi.yml \
34.0-rest-api.yml \
34.0-cloudapi.yml \
35.0-rest-api.yml \
35.2-rest-api.yml

.PHONY : all
all : $(specs)

website/27.0.zip:
	mkdir -p $(dir $@)
	curl https://vdc-repo.vmware.com/vmwb-repository/dcr-public/76f491b4-679c-4e1e-8428-f813d668297a/a2555a1b-22f1-4cca-b481-2a98ab874022/doc/a2555a1b-22f1-4cca-b481-2a98ab874022.zip > $@

website/29.0.zip:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/ca48e1bb-282b-4fdc-b827-649b819249ed/55142cf1-5bb8-4ab1-8d09-b84f717af5ec/doc/55142cf1-5bb8-4ab1-8d09-b84f717af5ec.zip > $@

website/30.0.zip:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/7a028e78-bd37-4a6a-8298-9c26c7eeb9aa/09142237-dd46-4dee-8326-e07212fb63a8/doc/09142237-dd46-4dee-8326-e07212fb63a8.zip > $@

website/31.0.zip:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/f27d65ea-c25b-45ed-9193-c8cc77507622/9a1f04e3-359b-4a19-9c62-7c0fafdfeac8/doc/9a1f04e3-359b-4a19-9c62-7c0fafdfeac8.zip > $@

website/32.0.zip:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/71e12563-bc11-4d64-821d-92d30f8fcfa1/7424bf8e-aec2-44ad-be7d-b98feda7bae0/doc/7424bf8e-aec2-44ad-be7d-b98feda7bae0.zip > $@

website/33.0.zip:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/037ccaee-649a-417e-b365-1331034fb28d/1f0fd9eb-0238-4af6-89b5-7e6636f29c65/doc/1f0fd9eb-0238-4af6-89b5-7e6636f29c65.zip > $@

website/34.0.zip:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/06a3b3da-4c6d-4984-b795-5d64081a4b10/8e47d46b-cfa7-4c06-8b81-4f5548da3102/doc/8e47d46b-cfa7-4c06-8b81-4f5548da3102.zip > $@

website/35.0.zip:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/e5392f68-0310-4bb0-9622-52adfe664c4c/8a8ba663-8f08-471c-9bc9-9f998696b9c0/doc/8a8ba663-8f08-471c-9bc9-9f998696b9c0.zip > $@

website/35.2.zip:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/ad96a8e3-043d-4e88-a0ba-87db0965b492/029c9ce7-e5fc-47c7-8003-f4bfa046e6db/doc/029c9ce7-e5fc-47c7-8003-f4bfa046e6db.zip > $@

website/32.0.html:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/5f403ebf-b14c-4a1c-be10-1539c02415d6/0101cd7b-ae5f-4db8-bda1-23318a5e7a48/vcd-openapi-docs.html > $@

website/33.0.html:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/772aa4c5-7e61-4d80-8432-b8e0d821c969/2747ec83-6aef-4560-b1d1-55ed9adc4e73/vcd-openapi-docs.html > $@

website/34.0.html:
	mkdir -p $(dir $@)
	curl https://vdc-download.vmware.com/vmwb-repository/dcr-public/a36f68c4-9f5a-4a63-894c-eb3840773fe7/b37fc25f-f5f3-442b-b3a6-d93d38132e06/vcd-openapi-docs.html > $@

./%-rest-api.json: website/%.zip
	(cd transformer-rest-api; cargo run --release) < $< > $@

./%-cloudapi.json: website/%.html transformer-cloudapi/transformer.js
	input=$< node transformer-cloudapi/transformer.js > $@

./%.yml: ./%.json
	yq --yaml-output < $< > $@

transformer-cloudapi/transformer.js: transformer-cloudapi/*.ts
	cd transformer-cloudapi; npm install && npm run compile