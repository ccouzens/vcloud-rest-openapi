specs = \
website/27.0.json \
website/29.0.json \
website/30.0.json \
website/31.0.json \
website/32.0.json \
website/33.0.json \
website/34.0.json

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

website/%.json: website/%.zip
	(cd transformer; cargo run --release) < $(addsuffix .zip,$(basename $@)) > $@