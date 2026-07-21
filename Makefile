# Makefile for helpers for CI and development

.PHONY: check
check: elixir rust nix

##########
# Elixir #
##########

.PHONY: elixir
elixir: elixir-check elixir-docs elixir-formatted  elixir-test

.PHONY: elixir-check
elixir-check:
	mix compile --warnings-as-errors

.PHONY: elixir-docs
elixir-docs:
	mix docs

.PHONY: elixir-formatted
elixir-formatted:
	mix format --check-formatted

.PHONY: elixir-test
elixir-test:
	mix test

.PHONY: elixir-setup
elixir-setup:
	mix deps.get

########
# Rust #
########

.PHONY: rust
rust: rust-check rust-formatted rust-lint

.PHONY: rust-check
rust-check:
	cargo check

.PHONY: rust-formatted
rust-formatted:
	cargo fmt --check

.PHONY: rust-lint
rust-lint:
	cargo clippy

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
