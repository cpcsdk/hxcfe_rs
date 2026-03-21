.PHONY: publish

publish:
	cargo publish -p hxcfe-sys --allow-dirty
	cargo publish -p hxcfe --allow-dirty
	cargo publish -p hxcfe_cli --allow-dirty
