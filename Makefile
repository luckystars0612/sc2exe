# sc2exe Makefile - Cross-compilation support
# Build from Linux (including Kali) or Windows

CARGO = cargo
RELEASE = --release

# Native build
.PHONY: build
build:
	$(CARGO) build $(RELEASE)

# Build for Windows x64 from Linux
.PHONY: win64
win64:
	$(CARGO) build $(RELEASE) --target x86_64-pc-windows-gnu

# Build for Windows x32 from Linux  
.PHONY: win32
win32:
	$(CARGO) build $(RELEASE) --target i686-pc-windows-gnu

# Build for Linux x64 from Windows
.PHONY: linux64
linux64:
	$(CARGO) build $(RELEASE) --target x86_64-unknown-linux-gnu

# Build for Linux x32 from Windows
.PHONY: linux32
linux32:
	$(CARGO) build $(RELEASE) --target i686-unknown-linux-gnu

# Build all targets
.PHONY: all
all: build win64 win32

# Clean
.PHONY: clean
clean:
	$(CARGO) clean

# Install targets (run once)
.PHONY: setup
setup:
	rustup target add x86_64-pc-windows-gnu
	rustup target add i686-pc-windows-gnu
	rustup target add x86_64-unknown-linux-gnu
	rustup target add i686-unknown-linux-gnu
	@echo ""
	@echo "On Kali/Debian, also run:"
	@echo "  sudo apt install mingw-w64"

# Help
.PHONY: help
help:
	@echo "sc2exe build targets:"
	@echo "  make build   - Native build"
	@echo "  make win64   - Windows x64 (cross-compile from Linux)"
	@echo "  make win32   - Windows x32 (cross-compile from Linux)"
	@echo "  make all     - Build native + Windows targets"
	@echo "  make setup   - Install cross-compilation toolchains"
	@echo "  make clean   - Clean build artifacts"