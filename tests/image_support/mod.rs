//! Port of mem_fd_helper.hpp and of the image builders of the scanner tests.

// Every test binary compiles this module and uses a different part of it
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

pub fn w16(buf: &mut [u8], off: usize, value: u16) {
    buf[off..off + 2].copy_from_slice(&value.to_le_bytes());
}

pub fn w32(buf: &mut [u8], off: usize, value: u32) {
    buf[off..off + 4].copy_from_slice(&value.to_le_bytes());
}

pub fn w64(buf: &mut [u8], off: usize, value: u64) {
    buf[off..off + 8].copy_from_slice(&value.to_le_bytes());
}

/// A uniquely named file in the temp directory, removed on drop.
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn create(data: &[u8]) -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);

        let name = format!("omath_test_{}_{}.bin", std::process::id(), NEXT_ID.fetch_add(1, Ordering::Relaxed));
        let path = std::env::temp_dir().join(name);

        std::fs::write(&path, data).expect("failed to write the temporary file");

        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// Layout of build_minimal_pe:
//   0x00: DOS header (64 B)   0x40: pad   0x80: NT sig   0x84: FileHeader (20 B)
//   0x98: OptionalHeader (0xF0 B)   0x188: SectionHeader (44 B)   0x1B4: section data
pub const PE_E_LFANEW: usize = 0x80;
pub const PE_SIZE_OPT: u16 = 0xF0;
pub const PE_FH_OFF: usize = PE_E_LFANEW + 4;
pub const PE_OH_OFF: usize = PE_FH_OFF + 20;
pub const PE_SH_OFF: usize = PE_OH_OFF + PE_SIZE_OPT as usize;
pub const PE_DATA_OFF: usize = PE_SH_OFF + 44;

/// Builds a minimal PE binary with a single section named `name`.
pub fn build_minimal_pe_with_section(section_bytes: &[u8], name: &[u8; 8]) -> Vec<u8> {
    let mut buf = vec![0u8; PE_DATA_OFF + section_bytes.len()];

    buf[0] = b'M';
    buf[1] = b'Z';
    w32(&mut buf, 0x3C, PE_E_LFANEW as u32);

    buf[PE_E_LFANEW] = b'P';
    buf[PE_E_LFANEW + 1] = b'E';

    w16(&mut buf, PE_FH_OFF, 0x8664); // machine
    w16(&mut buf, PE_FH_OFF + 2, 1); // num_sections
    w16(&mut buf, PE_FH_OFF + 16, PE_SIZE_OPT);

    w16(&mut buf, PE_OH_OFF, 0x20B); // magic

    buf[PE_SH_OFF..PE_SH_OFF + 8].copy_from_slice(name);

    let vsize = section_bytes.len() as u32;
    w32(&mut buf, PE_SH_OFF + 8, vsize);
    w32(&mut buf, PE_SH_OFF + 12, 0x1000); // vaddr
    w32(&mut buf, PE_SH_OFF + 16, vsize);
    w32(&mut buf, PE_SH_OFF + 20, PE_DATA_OFF as u32); // ptr_raw

    buf[PE_DATA_OFF..].copy_from_slice(section_bytes);
    buf
}

pub fn build_minimal_pe(section_bytes: &[u8]) -> Vec<u8> {
    build_minimal_pe_with_section(section_bytes, b".text\0\0\0")
}

/// A raw PE file where `ptr_raw_data` differs from `virtual_address`.
pub fn make_fake_pe_file(virtual_address: u32, ptr_raw_data: u32, section_size: u32, code_bytes: &[u8]) -> Vec<u8> {
    const E_LFANEW: usize = 0x80;
    const OPT_HDR_SIZE: u16 = 0xF0;
    const FH_OFF: usize = E_LFANEW + 4;
    const OPT_OFF: usize = FH_OFF + 20;
    const SH_OFF: usize = OPT_OFF + OPT_HDR_SIZE as usize;

    let mut buf = vec![0u8; (ptr_raw_data + section_size + 0x100) as usize];

    w16(&mut buf, 0x00, 0x5A4D);
    w32(&mut buf, 0x3C, E_LFANEW as u32);
    w32(&mut buf, E_LFANEW, 0x4550);

    w16(&mut buf, FH_OFF + 2, 1); // num_sections
    w16(&mut buf, FH_OFF + 16, OPT_HDR_SIZE);

    w16(&mut buf, OPT_OFF, 0x020B); // PE32+
    w64(&mut buf, OPT_OFF + 24, 0); // ImageBase = 0 to keep virtual_base_addr in 32-bit range

    buf[SH_OFF..SH_OFF + 5].copy_from_slice(b".text");
    w32(&mut buf, SH_OFF + 8, section_size); // VirtualSize
    w32(&mut buf, SH_OFF + 12, virtual_address); // VirtualAddress
    w32(&mut buf, SH_OFF + 16, section_size); // SizeOfRawData
    w32(&mut buf, SH_OFF + 20, ptr_raw_data); // PointerToRawData

    let copy_len = code_bytes.len().min(section_size as usize);
    let raw_off = ptr_raw_data as usize;
    buf[raw_off..raw_off + copy_len].copy_from_slice(&code_bytes[..copy_len]);

    buf
}

// Layout of make_elf64_with_text_section:
//   0x000 : ELF64 file header  (64 bytes)
//   0x040 : section data       (padded to 0x20 bytes)
//   0x060 : section name table "\0.text\0\0"
//   0x068 : section header table  (3 entries of 64 bytes)
pub const ELF_TEXT_OFF: usize = 0x40;
pub const ELF_TEXT_SIZE: usize = 0x20;
pub const ELF_SHSTRTAB_OFF: usize = ELF_TEXT_OFF + ELF_TEXT_SIZE;
pub const ELF_SHSTRTAB_SIZE: usize = 8;
pub const ELF_SHDR_TABLE_OFF: usize = ELF_SHSTRTAB_OFF + ELF_SHSTRTAB_SIZE;
pub const ELF_SHDR_SIZE: usize = 64;

/// Minimal ELF64 file with a single .text section containing known bytes.
pub fn make_elf64_with_text_section(code_bytes: &[u8]) -> Vec<u8> {
    const NUM_SECTIONS: usize = 3; // null + .text + .shstrtab

    let mut buf = vec![0u8; ELF_SHDR_TABLE_OFF + NUM_SECTIONS * ELF_SHDR_SIZE];

    buf[..4].copy_from_slice(b"\x7FELF");
    buf[4] = 2; // ELFCLASS64
    buf[5] = 1; // ELFDATA2LSB
    buf[6] = 1; // EV_CURRENT
    w16(&mut buf, 16, 2); // e_type = ET_EXEC
    w16(&mut buf, 18, 62); // e_machine = EM_X86_64
    w32(&mut buf, 20, 1); // e_version
    w64(&mut buf, 40, ELF_SHDR_TABLE_OFF as u64); // e_shoff
    w16(&mut buf, 52, 64); // e_ehsize
    w16(&mut buf, 54, 56); // e_phentsize
    w16(&mut buf, 58, ELF_SHDR_SIZE as u16); // e_shentsize
    w16(&mut buf, 60, NUM_SECTIONS as u16); // e_shnum
    w16(&mut buf, 62, 2); // e_shstrndx = 2 (.shstrtab is section index 2)

    let copy_len = code_bytes.len().min(ELF_TEXT_SIZE);
    buf[ELF_TEXT_OFF..ELF_TEXT_OFF + copy_len].copy_from_slice(&code_bytes[..copy_len]);

    buf[ELF_SHSTRTAB_OFF + 1..ELF_SHSTRTAB_OFF + 6].copy_from_slice(b".text");

    // Section 1: .text
    let base = ELF_SHDR_TABLE_OFF + ELF_SHDR_SIZE;
    w32(&mut buf, base, 1); // sh_name
    w32(&mut buf, base + 4, 1); // sh_type = SHT_PROGBITS
    w64(&mut buf, base + 8, 6); // sh_flags = SHF_ALLOC|SHF_EXECINSTR
    w64(&mut buf, base + 16, ELF_TEXT_OFF as u64); // sh_addr (same as offset in test)
    w64(&mut buf, base + 24, ELF_TEXT_OFF as u64); // sh_offset
    w64(&mut buf, base + 32, ELF_TEXT_SIZE as u64); // sh_size
    w64(&mut buf, base + 48, 16); // sh_addralign

    // Section 2: .shstrtab
    let base = ELF_SHDR_TABLE_OFF + 2 * ELF_SHDR_SIZE;
    w32(&mut buf, base + 4, 3); // sh_type = SHT_STRTAB
    w64(&mut buf, base + 24, ELF_SHSTRTAB_OFF as u64); // sh_offset
    w64(&mut buf, base + 32, ELF_SHSTRTAB_SIZE as u64); // sh_size

    buf
}

pub const MH_MAGIC_64: u32 = 0xFEEDFACF;
pub const MH_MAGIC_32: u32 = 0xFEEDFACE;
pub const LC_SEGMENT: u32 = 0x1;
pub const LC_SEGMENT_64: u32 = 0x19;

pub const MACHO64_HDR_SIZE: usize = 32;
pub const MACHO64_SEG_SIZE: usize = 72;
pub const MACHO64_SECT_SIZE: usize = 80;
pub const MACHO32_HDR_SIZE: usize = 28;
pub const MACHO32_SEG_SIZE: usize = 56;
pub const MACHO32_SECT_SIZE: usize = 68;

const SEGMENT_NAME: &[u8] = b"__TEXT";
const SECTION_NAME: &[u8] = b"__text";

/// Minimal 64-bit Mach-O binary with a __text section at virtual address `addr`.
fn macho64(section_bytes: &[u8], addr: u64, cputype: u32) -> Vec<u8> {
    const LOAD_CMD_SIZE: usize = MACHO64_SEG_SIZE + MACHO64_SECT_SIZE;
    const SECTION_OFFSET: usize = MACHO64_HDR_SIZE + LOAD_CMD_SIZE;

    let mut buf = vec![0u8; SECTION_OFFSET + section_bytes.len()];
    let size = section_bytes.len() as u64;

    w32(&mut buf, 0, MH_MAGIC_64);
    w32(&mut buf, 4, cputype);
    w32(&mut buf, 8, 0x3); // cpusubtype
    w32(&mut buf, 12, 0x2); // filetype = MH_EXECUTE
    w32(&mut buf, 16, 1); // ncmds
    w32(&mut buf, 20, LOAD_CMD_SIZE as u32); // sizeofcmds

    let seg = MACHO64_HDR_SIZE;
    w32(&mut buf, seg, LC_SEGMENT_64);
    w32(&mut buf, seg + 4, LOAD_CMD_SIZE as u32);
    buf[seg + 8..seg + 8 + SEGMENT_NAME.len()].copy_from_slice(SEGMENT_NAME);
    w64(&mut buf, seg + 24, addr); // vmaddr
    w64(&mut buf, seg + 32, size); // vmsize
    w64(&mut buf, seg + 40, SECTION_OFFSET as u64); // fileoff
    w64(&mut buf, seg + 48, size); // filesize
    w32(&mut buf, seg + 56, 7); // maxprot
    w32(&mut buf, seg + 60, 5); // initprot
    w32(&mut buf, seg + 64, 1); // nsects

    let sect = seg + MACHO64_SEG_SIZE;
    buf[sect..sect + SECTION_NAME.len()].copy_from_slice(SECTION_NAME);
    buf[sect + 16..sect + 16 + SEGMENT_NAME.len()].copy_from_slice(SEGMENT_NAME);
    w64(&mut buf, sect + 32, addr);
    w64(&mut buf, sect + 40, size);
    w32(&mut buf, sect + 48, SECTION_OFFSET as u32);

    buf[SECTION_OFFSET..].copy_from_slice(section_bytes);
    buf
}

pub fn build_minimal_macho64(section_bytes: &[u8]) -> Vec<u8> {
    macho64(section_bytes, 0x1_0000_0000, 0x01000007) // CPU_TYPE_X86_64
}

/// Minimal 32-bit Mach-O binary with a __text section.
pub fn build_minimal_macho32(section_bytes: &[u8]) -> Vec<u8> {
    const LOAD_CMD_SIZE: usize = MACHO32_SEG_SIZE + MACHO32_SECT_SIZE;
    const SECTION_OFFSET: usize = MACHO32_HDR_SIZE + LOAD_CMD_SIZE;

    let mut buf = vec![0u8; SECTION_OFFSET + section_bytes.len()];
    let size = section_bytes.len() as u32;

    w32(&mut buf, 0, MH_MAGIC_32);
    w32(&mut buf, 4, 0x7); // cputype
    w32(&mut buf, 8, 0x3); // cpusubtype
    w32(&mut buf, 12, 0x2); // filetype
    w32(&mut buf, 16, 1); // ncmds
    w32(&mut buf, 20, LOAD_CMD_SIZE as u32); // sizeofcmds

    let seg = MACHO32_HDR_SIZE;
    w32(&mut buf, seg, LC_SEGMENT);
    w32(&mut buf, seg + 4, LOAD_CMD_SIZE as u32);
    buf[seg + 8..seg + 8 + SEGMENT_NAME.len()].copy_from_slice(SEGMENT_NAME);
    w32(&mut buf, seg + 24, 0x1000); // vmaddr
    w32(&mut buf, seg + 28, size); // vmsize
    w32(&mut buf, seg + 32, SECTION_OFFSET as u32); // fileoff
    w32(&mut buf, seg + 36, size); // filesize
    w32(&mut buf, seg + 40, 7); // maxprot
    w32(&mut buf, seg + 44, 5); // initprot
    w32(&mut buf, seg + 48, 1); // nsects

    let sect = seg + MACHO32_SEG_SIZE;
    buf[sect..sect + SECTION_NAME.len()].copy_from_slice(SECTION_NAME);
    buf[sect + 16..sect + 16 + SEGMENT_NAME.len()].copy_from_slice(SEGMENT_NAME);
    w32(&mut buf, sect + 32, 0x1000); // addr
    w32(&mut buf, sect + 36, size);
    w32(&mut buf, sect + 40, SECTION_OFFSET as u32);

    buf[SECTION_OFFSET..].copy_from_slice(section_bytes);
    buf
}

/// Mach-O 64-bit file with a single __text section padded to 0x20 bytes.
pub fn make_macho64_with_text_section(code_bytes: &[u8]) -> Vec<u8> {
    let mut section = vec![0u8; 0x20];
    let copy_len = code_bytes.len().min(section.len());
    section[..copy_len].copy_from_slice(&code_bytes[..copy_len]);

    macho64(&section, 0x1000, 0x0100000C) // CPU_TYPE_ARM64
}
