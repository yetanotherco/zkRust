install: install_risc0 install_sp1

install_risc0:
	@curl -L https://risczero.com/install | bash
	@rzup
	@cargo risczero --version
	@echo "Risc0 Toolchain Installed"

install_sp1:
	@curl -L https://sp1.succinct.xyz | bash
	@sp1up
	@cargo prove --version
	@echo "Sp1 Toolchain Installed"

# Default target
all: install
