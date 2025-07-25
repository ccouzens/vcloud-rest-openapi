specs = \
39.0.yml \
39.1.yml 

.NOTINTERMEDIATE: ./%.json

.PHONY : all
all : $(specs)

website/%.zip:
	./scripts/mirror $@

./%.json: website/%.zip
	(cd transformer; RUST_LOG=transformer=debug cargo run --release) < $< > $@

./%.yml: ./%.json
	yq -P '.' --output-format=yaml $< > $@

.PHONY : clean
clean :
	rm -f *.json *.yml
