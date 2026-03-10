# Makefile for helpers for CI and development

.PHONY: check
check: rust nix

########
# Rust #
########

.PHONY: rust
rust: rust-check rust-formatted rust-lint

NATIVE := turxo/native/turxo_nif/

.PHONY: rust-check
rust-check:
	cd $(NATIVE) && cargo check

.PHONY: rust-formatted
rust-formatted:
	cd $(NATIVE) && cargo fmt --check

.PHONY: rust-lint
rust-lint:
	cd $(NATIVE) && cargo clippy

#######
# Nix #
#######

.PHONY: nix
nix: nix-check nix-formatted nix-devShell

.PHONY: nix-check
nix-check: flake.nix
	nix flake check --all-systems

.PHONY: nix-formatted
nix-formatted: flake.nix
	nix develop --command nixfmt -c flake.nix

.PHONY: nix-devShell
nix-devShell: flake.nix
	nix develop --command echo test
