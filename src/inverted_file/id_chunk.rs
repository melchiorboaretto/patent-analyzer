pub const PAGE_SIZE: u64 = 512;
const MAX_IDS_PER_CHUNK: u64 = ((PAGE_SIZE as usize - (size_of::<u64>() + size_of::<i64>())) / size_of::<u64>()) as u64;

use std::io::Result;

use positioned_io::{
    RandomAccessFile, ReadAt,
};

use crate::inverted_file::InvertedIndex;

#[repr(C)]
pub struct IdChunk {
    pub len: u64, // Here I used u64 for padding reasons.
    pub next: i64, // If next == -1, there's no next
    pub content: [u64; MAX_IDS_PER_CHUNK as usize],
}

impl IdChunk {

    /// Creates a blank IdChunk
    pub fn new() -> Self {
        IdChunk {
            len: 0,
            next: -1,
            content: [0; MAX_IDS_PER_CHUNK as usize],
        }
    }

    /// READS an IdChunk instead of creating a blank one
    pub fn read_new(file: &RandomAccessFile, index: u64) -> Result<Self> {
        unsafe {
            let mut bytes = [0; PAGE_SIZE as usize];

            file.read_exact_at(InvertedIndex::offset(index), &mut bytes)?;
            Ok(IdChunk::from_bytes(&bytes))
        }
    }

    /// Tries to insert the given ID into the chunk, returns true if it could.
    /// Else, returns false.
    /// It WON'T adjust any next pointers, this must be done manually.
    fn try_insert(&mut self, id: u64) -> bool {
        if self.len == MAX_IDS_PER_CHUNK {
            false
        } else {
            self.content[self.len()] = id;
            self.len += 1;
            true
        }

    }

    /// adjust the next field of a chunk. 
    pub fn next_is(&mut self, index: u64) {
        self.next = index as i64;
    }

    /// Insert all the given indexes into the IdChunk, consuming it and returning a vector with all
    /// the chunks needed for the exceeding values. The first chunk of the vector is always the
    /// consumed one.
    ///
    /// NONE OF THE NEXT POINTERS ARE ADJUSTED. IT MUST BE DONE WHEN INSERTING THEM INTO THE FILE.
    pub fn insert_all(self, ids: Vec<u64>) -> Vec<IdChunk> {

        let mut chunk = self;
        let mut return_chunks = Vec::with_capacity(ids.len().div_ceil(MAX_IDS_PER_CHUNK as usize));

        for item in ids {

            if !chunk.try_insert(item) {
                return_chunks.push(chunk);
                chunk = IdChunk::new();

                chunk.try_insert(item); // This is always true because the chunk is new.
            }

        }

        return_chunks.push(chunk);

        return_chunks

    }

    /// Converts an IdChunk to its corresponding bytes, ready to be written in a binary file.
    ///
    /// The unsafe call is not bad because of the constant C like ABI, so that the fiels are always
    /// in the correct order.
    pub fn as_bytes(&self) -> &[u8] {

        unsafe {
            let raw_ptr = self as *const IdChunk as *const u8; 

            std::slice::from_raw_parts(raw_ptr, size_of::<Self>())
        }

    }

    /// The function itself is unsafe because the caller must know that the file is not corrupted
    /// and the byte sequence is exactly PAGE_SIZE Bytes long.
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let raw_ptr = bytes.as_ptr() as *const Self;

        unsafe {
            std::ptr::read_unaligned(raw_ptr)
        }
    }

    /// Give the length of a chunk with a "size" type, easier to deal with.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn next(&self) -> u64 {
        self.next as u64
    }

}
