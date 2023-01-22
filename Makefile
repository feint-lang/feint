stamp = $(shell ls -t target/release/build/feint-*/out/feint.stamp | head -n 1)
bash_completions_dir = ~/.local/share/bash-completion/completions
fish_completions_dir = ~/.config/fish/completions

.PHONY = install
install:
	@echo "Building FeInt and installing to ~/.cargo..."
	cargo install --root ~/.cargo --path .

.PHONY = install-bash-completion
install-bash-completion:
	mkdir -p $(bash_completions_dir)
	cp $(shell dirname $(stamp))/feint.bash $(bash_completions_dir)/feint

.PHONY = install-fish-completion
install-fish-completion:
	mkdir -p $(fish_completions_dir)
	cp $(shell dirname $(stamp))/feint.fish $(fish_completions_dir)/feint.fish

.PHONY = docs
docs:
	@echo "Building Cargo docs..."
	cargo doc
	@echo
	@echo "Building other docs..."
	cd doc && mdbook build
