# Makefile for helpers for CI and development

.PHONY: check
check: elixir-turxo elixir-ecto_turxo rust nix

##########
# Elixir #
##########

.PHONY: elixir-%
elixir-%: elixir-%-check elixir-%-formatted  elixir-%-test  

.PHONY: elixir-%-check
elixir-%-check:
	cd $* && mix compile --warnings-as-errors

.PHONY: elixir-%-formatted
elixir-%-formatted:
	cd $* && mix format --check-formatted

.PHONY: elixir-%-test
elixir-%-test:
	cd $* && mix test

.PHONY: elixir-%-setup
elixir-%-setup: 
	cd $* && mix deps.get

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
