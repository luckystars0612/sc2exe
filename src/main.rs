use clap::{Parser, ValueEnum};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod pe;
mod elf;

#[derive(Parser)]
#[command(name = "sc2exe")]
#[command(about = "Convert shellcode to executable for IDA Pro debugging")]
#[command(version)]
struct Args {
    /// Input shellcode file
    #[arg(short = 'f', long)]
    file: PathBuf,

    /// Output executable path
    #[arg(short, long)]
    output: PathBuf,

    /// Target format
    #[arg(short, long, value_enum, default_value = "win64")]
    target: Target,

    /// Add int3 breakpoint before shellcode
    #[arg(short, long, default_value = "true")]
    pause: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
enum Target {
    Win64,
    Win32,
    Linux64,
    Linux32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let shellcode = fs::read(&args.file)?;
    println!("[*] Read {} bytes from {:?}", shellcode.len(), args.file);

    let exe_data = match args.target {
        Target::Win64 => pe::build_pe(&shellcode, true, args.pause)?,
        Target::Win32 => pe::build_pe(&shellcode, false, args.pause)?,
        Target::Linux64 => elf::build_elf(&shellcode, true, args.pause)?,
        Target::Linux32 => elf::build_elf(&shellcode, false, args.pause)?,
    };

    let mut file = fs::File::create(&args.output)?;
    file.write_all(&exe_data)?;

    #[cfg(unix)]
    if matches!(args.target, Target::Linux64 | Target::Linux32) {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms)?;
    }

    println!("[+] Created {:?} ({} bytes)", args.output, exe_data.len());
    println!("[+] Target: {:?}", args.target);
    println!("[+] Breakpoint: {}", if args.pause { "yes (int3)" } else { "no" });

    Ok(())
}