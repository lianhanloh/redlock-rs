build:
	@cargo build

docs: build
	cargo rustdoc -- --no-defaults --passes collapse-docs --passes unindent-comments

upload-docs: docs
		@./upload-docs.sh

.PHONY: build docs upload-docs
