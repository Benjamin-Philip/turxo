# Makefile for helpers for CI and development

.PHONY: check
check: nix

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
