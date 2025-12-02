# sc2exe

Convert shellcode to standalone executables for IDA Pro / x64dbg / GDB debugging.

## Features

- Generate Windows PE (32/64-bit) and Linux ELF (32/64-bit)
- Cross-compile: build Windows .exe from Kali Linux (or vice versa)
- Optional `int3` breakpoint before shellcode
- Dedicated `.shell` section for easy analysis

## Quick Start

```bash
# Build sc2exe
cargo build --release

# Convert shellcode to Windows exe
./target/release/sc2exe -f shellcode.bin -o payload.exe

# Convert to Linux ELF
./target/release/sc2exe -f shellcode.bin -o payload -t linux64
```

## Installation

### On Kali Linux / Debian

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install MinGW for Windows cross-compilation
sudo apt update
sudo apt install mingw-w64

# Add Windows targets
rustup target add x86_64-pc-windows-gnu
rustup target add i686-pc-windows-gnu

# Build
cargo build --release
```

### On Windows

```powershell
# Install Rust from https://rustup.rs

# Build
cargo build --release
```

## Cross-Compilation

### Build Windows .exe from Kali Linux

```bash
# One-time setup
make setup
sudo apt install mingw-w64

# Build Windows 64-bit sc2exe.exe
make win64

# Output: target/x86_64-pc-windows-gnu/release/sc2exe.exe
```

### Build Linux binary from Windows

Use WSL or a Linux cross-compiler.

## Usage

```bash
sc2exe -f <shellcode> -o <output> [OPTIONS]

OPTIONS:
  -f, --file <FILE>      Input shellcode file
  -o, --output <OUTPUT>  Output executable
  -t, --target <TARGET>  Target: win64, win32, linux64, linux32 [default: win64]
  -p, --pause <PAUSE>    Add int3 breakpoint [default: true]
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

```bash
# Windows 64-bit with breakpoint (default)
sc2exe -f calc.bin -o calc.exe

# Windows 32-bit
sc2exe -f calc.bin -o calc.exe -t win32

# Linux 64-bit
sc2exe -f shell.bin -o shell -t linux64

# Without breakpoint
sc2exe -f calc.bin -o calc.exe -p false
```

## Debugging in IDA Pro

1. Open the generated `.exe` in IDA Pro
2. Go to **View → Open subviews → Segments** (Shift+F7)
3. Find `.shell` section - this contains your shellcode
4. Set **Debugger → Select debugger → Local Windows debugger**
5. Press **F9** to run - it breaks at `int3`
6. Press **F7** to step into shellcode

## Generating Test Shellcode

```bash
# Windows calc.exe (64-bit)
msfvenom -p windows/x64/exec CMD=calc.exe -f raw -o calc.bin

# Windows MessageBox (64-bit)
msfvenom -p windows/x64/messagebox TEXT=Hello -f raw -o msg.bin

# Linux /bin/sh (64-bit)
msfvenom -p linux/x64/exec CMD=/bin/sh -f raw -o sh.bin
```

## Project Structure

```
sc2exe/
├── Cargo.toml
├── Makefile
├── .cargo/
│   └── config.toml    # Cross-compilation linkers
└── src/
    ├── main.rs        # CLI
    ├── pe.rs          # Windows PE builder
    └── elf.rs         # Linux ELF builder
```

## Make Targets

```bash
make build   # Native build
make win64   # Windows x64 from Linux
make win32   # Windows x32 from Linux
make all     # Native + Windows
make setup   # Install toolchains
make clean   # Clean
```