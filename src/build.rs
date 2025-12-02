// build.rs - Optional build script for custom section
// This creates a custom .shell section for the shellcode

fn main() {
    // For MSVC linker (Windows)
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=/SECTION:.shell,ERW");
    }
    
    // Prevent rerunning unless build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}