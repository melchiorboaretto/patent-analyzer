pub const METADATA_PAGES: u64 = 1;
const METADATA_FIELDS: u64 = 3;

use super::id_chunk::PAGE_SIZE;

#[repr(C)]
pub struct Metadata {
    pub file_len: u64,
    pub recycled: u64,
    pub available: u64, // If available == 0, the idea is to auto-append, instead of applying some
    // best-fit algorithm
    pub free_idx: [u64; (PAGE_SIZE/8 - METADATA_FIELDS) as usize],
}

impl Metadata {

    pub fn new() -> Self {
        Metadata {
            file_len: 0,
            recycled: 0,
            available: 0,
            free_idx: [0; (PAGE_SIZE/8 - METADATA_FIELDS) as usize]
        }
    }

    pub fn as_bytes(&self) -> &[u8] {

        unsafe {
            let raw_ptr = self as *const Metadata as *const u8; 

            std::slice::from_raw_parts(raw_ptr, size_of::<Self>())
        }

    }

    /// The function itself is unsafe because the caller must know that the file is not corrupted
    /// and the byte sequence is exactly PAGE_SIZE Bytes long.
    pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let raw_ptr = bytes.as_ptr() as *const Self;

        unsafe {
            std::ptr::read_unaligned(raw_ptr)
        }
    }

}
