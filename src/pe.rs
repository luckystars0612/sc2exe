use scroll::{Pwrite, LE};

const FILE_ALIGN: usize = 0x200;
const SECT_ALIGN: usize = 0x1000;

pub fn build_pe(sc: &[u8], is64: bool, pause: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (opt_hdr_sz, img_base): (u16, u64) = if is64 { (240, 0x140000000) } else { (224, 0x400000) };
    
    let stub = make_stub(is64, pause);
    let hdr_sz = align(64 + 4 + 20 + opt_hdr_sz as usize + 80, FILE_ALIGN);
    let text_raw = align(stub.len(), FILE_ALIGN);
    let text_va = SECT_ALIGN;
    let shell_raw = align(sc.len(), FILE_ALIGN);
    let shell_va = text_va + align(stub.len(), SECT_ALIGN);
    let img_sz = shell_va + align(sc.len(), SECT_ALIGN);
    
    let mut b = vec![0u8; hdr_sz + text_raw + shell_raw];
    let mut o = 0;

    // DOS Header
    b.gwrite_with::<u16>(0x5A4D, &mut o, LE)?;
    o = 0x3C;
    b.gwrite_with::<u32>(64, &mut o, LE)?;

    // PE Signature
    o = 64;
    b.gwrite_with::<u32>(0x4550, &mut o, LE)?;

    // COFF Header
    b.gwrite_with::<u16>(if is64 { 0x8664 } else { 0x14C }, &mut o, LE)?;
    b.gwrite_with::<u16>(2, &mut o, LE)?; // sections
    o += 12;
    b.gwrite_with::<u16>(opt_hdr_sz, &mut o, LE)?;
    b.gwrite_with::<u16>(if is64 { 0x22 } else { 0x122 }, &mut o, LE)?;

    // Optional Header
    b.gwrite_with::<u16>(if is64 { 0x20B } else { 0x10B }, &mut o, LE)?;
    b.gwrite_with::<u16>(0, &mut o, LE)?; // linker ver
    b.gwrite_with::<u32>(text_raw as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(shell_raw as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(0, &mut o, LE)?;
    b.gwrite_with::<u32>(text_va as u32, &mut o, LE)?; // entry
    b.gwrite_with::<u32>(text_va as u32, &mut o, LE)?; // base of code

    if is64 {
        b.gwrite_with::<u64>(img_base, &mut o, LE)?;
    } else {
        b.gwrite_with::<u32>(shell_va as u32, &mut o, LE)?;
        b.gwrite_with::<u32>(img_base as u32, &mut o, LE)?;
    }

    b.gwrite_with::<u32>(SECT_ALIGN as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(FILE_ALIGN as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(6, &mut o, LE)?; // OS ver
    b.gwrite_with::<u32>(0, &mut o, LE)?; // img ver
    b.gwrite_with::<u32>(6, &mut o, LE)?; // subsys ver
    b.gwrite_with::<u32>(0, &mut o, LE)?; // win32 ver
    b.gwrite_with::<u32>(img_sz as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(hdr_sz as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(0, &mut o, LE)?; // checksum
    b.gwrite_with::<u16>(3, &mut o, LE)?; // console
    b.gwrite_with::<u16>(0x8160, &mut o, LE)?; // dll chars

    if is64 {
        for _ in 0..4 { b.gwrite_with::<u64>(0x10000, &mut o, LE)?; }
    } else {
        for _ in 0..4 { b.gwrite_with::<u32>(0x10000, &mut o, LE)?; }
    }
    b.gwrite_with::<u32>(0, &mut o, LE)?;
    b.gwrite_with::<u32>(16, &mut o, LE)?;
    for _ in 0..16 { b.gwrite_with::<u64>(0, &mut o, LE)?; }

    // Section: .text
    b[o..o+8].copy_from_slice(b".text\0\0\0"); o += 8;
    b.gwrite_with::<u32>(stub.len() as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(text_va as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(text_raw as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(hdr_sz as u32, &mut o, LE)?;
    o += 12;
    b.gwrite_with::<u32>(0x60000020, &mut o, LE)?;

    // Section: .shell
    b[o..o+8].copy_from_slice(b".shell\0\0"); o += 8;
    b.gwrite_with::<u32>(sc.len() as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(shell_va as u32, &mut o, LE)?;
    b.gwrite_with::<u32>(shell_raw as u32, &mut o, LE)?;
    b.gwrite_with::<u32>((hdr_sz + text_raw) as u32, &mut o, LE)?;
    o += 12;
    b.gwrite_with::<u32>(0xE0000060, &mut o, LE)?;

    // Patch stub with jump offset to .shell
    let mut stub = stub;
    let jmp_off = (shell_va as i32) - (text_va as i32 + stub.len() as i32);
    let idx = stub.len() - 4;
    stub[idx..].copy_from_slice(&jmp_off.to_le_bytes());

    b[hdr_sz..hdr_sz + stub.len()].copy_from_slice(&stub);
    b[hdr_sz + text_raw..hdr_sz + text_raw + sc.len()].copy_from_slice(sc);

    Ok(b)
}

fn make_stub(is64: bool, pause: bool) -> Vec<u8> {
    let mut v = Vec::new();
    if pause { v.push(0xCC); } // int3
    if is64 {
        // lea rax, [rip+X]; jmp rax  (X patched later)
        v.extend_from_slice(&[0x48, 0x8D, 0x05]);
        v.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // rel32 placeholder
        v.extend_from_slice(&[0xFF, 0xE0]);
        // recalc: jmp rel32 is simpler
        v.clear();
        if pause { v.push(0xCC); }
        v.push(0xE9);
        v.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    } else {
        v.push(0xE9);
        v.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    }
    v
}

fn align(v: usize, a: usize) -> usize { (v + a - 1) & !(a - 1) }