use scroll::{Pwrite, LE};

const PAGE: usize = 0x1000;

pub fn build_elf(sc: &[u8], is64: bool, pause: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if is64 { build_elf64(sc, pause) } else { build_elf32(sc, pause) }
}

fn build_elf64(sc: &[u8], pause: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let base: u64 = 0x400000;
    let ehdr_sz = 64;
    let phdr_sz = 56;
    let stub = make_stub64(pause);
    
    let hdr_end = ehdr_sz + phdr_sz * 2;
    let text_off = align(hdr_end, PAGE);
    let shell_off = text_off + align(stub.len(), PAGE);
    let total = shell_off + sc.len();
    
    let text_va = base + text_off as u64;
    let shell_va = base + shell_off as u64;
    
    let mut b = vec![0u8; total];
    let mut o = 0;

    // ELF Header
    b[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']); o = 4;
    b.gwrite_with::<u8>(2, &mut o, LE)?; // 64-bit
    b.gwrite_with::<u8>(1, &mut o, LE)?; // LE
    b.gwrite_with::<u8>(1, &mut o, LE)?; // version
    o = 16;
    b.gwrite_with::<u16>(2, &mut o, LE)?; // ET_EXEC
    b.gwrite_with::<u16>(0x3E, &mut o, LE)?; // x86_64
    b.gwrite_with::<u32>(1, &mut o, LE)?;
    b.gwrite_with::<u64>(text_va, &mut o, LE)?; // entry
    b.gwrite_with::<u64>(ehdr_sz as u64, &mut o, LE)?; // phoff
    b.gwrite_with::<u64>(0, &mut o, LE)?; // shoff
    b.gwrite_with::<u32>(0, &mut o, LE)?; // flags
    b.gwrite_with::<u16>(ehdr_sz as u16, &mut o, LE)?;
    b.gwrite_with::<u16>(phdr_sz as u16, &mut o, LE)?;
    b.gwrite_with::<u16>(2, &mut o, LE)?; // phnum
    b.gwrite_with::<u16>(0, &mut o, LE)?;
    b.gwrite_with::<u16>(0, &mut o, LE)?;
    b.gwrite_with::<u16>(0, &mut o, LE)?;

    // PHDR 1: .text
    o = ehdr_sz;
    b.gwrite_with::<u32>(1, &mut o, LE)?; // PT_LOAD
    b.gwrite_with::<u32>(5, &mut o, LE)?; // PF_R|PF_X
    b.gwrite_with::<u64>(text_off as u64, &mut o, LE)?;
    b.gwrite_with::<u64>(text_va, &mut o, LE)?;
    b.gwrite_with::<u64>(text_va, &mut o, LE)?;
    b.gwrite_with::<u64>(stub.len() as u64, &mut o, LE)?;
    b.gwrite_with::<u64>(stub.len() as u64, &mut o, LE)?;
    b.gwrite_with::<u64>(PAGE as u64, &mut o, LE)?;

    // PHDR 2: .shell
    b.gwrite_with::<u32>(1, &mut o, LE)?;
    b.gwrite_with::<u32>(7, &mut o, LE)?; // RWX
    b.gwrite_with::<u64>(shell_off as u64, &mut o, LE)?;
    b.gwrite_with::<u64>(shell_va, &mut o, LE)?;
    b.gwrite_with::<u64>(shell_va, &mut o, LE)?;
    b.gwrite_with::<u64>(sc.len() as u64, &mut o, LE)?;
    b.gwrite_with::<u64>(sc.len() as u64, &mut o, LE)?;
    b.gwrite_with::<u64>(PAGE as u64, &mut o, LE)?;

    // Patch and write stub
    let mut stub = stub;
    let jmp_off = (shell_va as i64 - (text_va as i64 + stub.len() as i64)) as i32;
    let idx = stub.len() - 4;
    stub[idx..].copy_from_slice(&jmp_off.to_le_bytes());
    b[text_off..text_off + stub.len()].copy_from_slice(&stub);

    // Write shellcode
    b[shell_off..shell_off + sc.len()].copy_from_slice(sc);

    Ok(b)
}

fn build_elf32(sc: &[u8], pause: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let base: u32 = 0x08048000;
    let ehdr_sz = 52;
    let phdr_sz = 32;
    let stub = make_stub32(pause);
    
    let hdr_end = ehdr_sz + phdr_sz * 2;
    let text_off = align(hdr_end, PAGE);
    let shell_off = text_off + align(stub.len(), PAGE);
    let total = shell_off + sc.len();
    
    let text_va = base + text_off as u32;
    let shell_va = base + shell_off as u32;
    
    let mut b = vec![0u8; total];
    let mut o = 0;

    // ELF Header
    b[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']); o = 4;
    b.gwrite_with::<u8>(1, &mut o, LE)?; // 32-bit
    b.gwrite_with::<u8>(1, &mut o, LE)?;
    b.gwrite_with::<u8>(1, &mut o, LE)?;
    o = 16;
    b.gwrite_with::<u16>(2, &mut o, LE)?;
    b.gwrite_with::<u16>(3, &mut o, LE)?; // i386
    b.gwrite_with::<u32>(1, &mut o, LE)?;
    b.gwrite_with::<u32>(text_va, &mut o, LE)?;
    b.gwrite_with::<u32>(ehdr_sz as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(0, &mut o, LE)?;
    b.gwrite_with::<u32>(0, &mut o, LE)?;
    b.gwrite_with::<u16>(ehdr_sz as u16, &mut o, LE)?;
    b.gwrite_with::<u16>(phdr_sz as u16, &mut o, LE)?;
    b.gwrite_with::<u16>(2, &mut o, LE)?;
    b.gwrite_with::<u16>(0, &mut o, LE)?;
    b.gwrite_with::<u16>(0, &mut o, LE)?;
    b.gwrite_with::<u16>(0, &mut o, LE)?;

    // PHDR 1
    o = ehdr_sz;
    b.gwrite_with::<u32>(1, &mut o, LE)?;
    b.gwrite_with::<u32>(text_off as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(text_va, &mut o, LE)?;
    b.gwrite_with::<u32>(text_va, &mut o, LE)?;
    b.gwrite_with::<u32>(stub.len() as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(stub.len() as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(5, &mut o, LE)?;
    b.gwrite_with::<u32>(PAGE as u32, &mut o, LE)?;

    // PHDR 2
    b.gwrite_with::<u32>(1, &mut o, LE)?;
    b.gwrite_with::<u32>(shell_off as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(shell_va, &mut o, LE)?;
    b.gwrite_with::<u32>(shell_va, &mut o, LE)?;
    b.gwrite_with::<u32>(sc.len() as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(sc.len() as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(7, &mut o, LE)?;
    b.gwrite_with::<u32>(PAGE as u32, &mut o, LE)?;

    let mut stub = stub;
    let jmp_off = (shell_va as i32) - (text_va as i32 + stub.len() as i32);
    let idx = stub.len() - 4;
    stub[idx..].copy_from_slice(&jmp_off.to_le_bytes());
    b[text_off..text_off + stub.len()].copy_from_slice(&stub);
    b[shell_off..shell_off + sc.len()].copy_from_slice(sc);

    Ok(b)
}

fn make_stub64(pause: bool) -> Vec<u8> {
    let mut v = Vec::new();
    if pause { v.push(0xCC); }
    v.push(0xE9);
    v.extend_from_slice(&[0; 4]);
    v
}

fn make_stub32(pause: bool) -> Vec<u8> {
    make_stub64(pause) // same opcode
}

fn align(v: usize, a: usize) -> usize { (v + a - 1) & !(a - 1) }