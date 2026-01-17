
const PAGE_SIZE: u64 = 512;
const MAX_IDS_PER_CHUNK: u64 = ((PAGE_SIZE as usize - (size_of::<u64>() + size_of::<i64>())) / size_of::<u64>()) as u64;

use std::fs;

use positioned_io::{
    ReadAt,
    RandomAccessFile,
};

pub struct InvertedIndex {
    path: String,
}

impl InvertedIndex {

    /// Initializes the InvertedIndex using a known file path. If the file does not exist
    /// it will be created, if it does, nothing more happens.
    pub fn new(path: String) -> Result<InvertedIndex, InvertedIndex> {
        match fs::OpenOptions::new()
            .create_new(true)
            .open(&path) {
            Ok(_) => {
                Ok(InvertedIndex {
                    path
                })
            },
            // NOTE: First error not handled
            Err(_) => {
                Err(InvertedIndex {
                    path
                })
            },
        }
    }

    /// Aux function to read a block of the inverted index, appends it to a vector.
    /// It is better if the vector is already initialized with the right size to avoid
    /// syscalls asking more memory
    fn read_block_to_vec(&self, ids: &mut Vec<u64>, file: &RandomAccessFile, index: u64) -> Option<u64> {

        // NOTE: Second error not handled
        let chunk = unsafe {
            let mut bytes = [0; PAGE_SIZE as usize];

            file.read_at(index * PAGE_SIZE, &mut bytes)
                .expect("ERROR READING CHUNK");
            IdChunk::from_bytes(&bytes)
        };

        ids.extend_from_slice(&chunk.content[0..chunk.len()]);

        if chunk.next != -1 {
            Some(chunk.next as u64)
        } else {
            None
        }
    }

    /// Retrieves all the ids for a given index
    pub fn get_ids(&self, mut index: u64) -> Vec<u64> {

        // NOTE: Third error not handled
        let file = RandomAccessFile::open(&self.path)
            .expect("ERROR OPENING FILE");

        let mut ids_vector = Vec::new();

        loop {

            let next = self.read_block_to_vec(&mut ids_vector, &file, index);

            if let Some(next_offset) = next {
                index = next_offset;
            } else {
                break;
            }
        }

        ids_vector
    }

}

#[repr(C)]
struct IdChunk {
    len: u64, // Here I used u64 for padding reasons.
    next: i64, // If next == -1, there's no next
    content: [u64; MAX_IDS_PER_CHUNK as usize],
}

impl IdChunk {

    /// Converts an IdChunk to its corresponding bytes, ready to be written in a binary file.
    ///
    /// The unsafe call is not bad because of the constant C like ABI, so that the fiels are always
    /// in the correct order.
    fn as_bytes(&self) -> &[u8] {

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
    fn len(&self) -> usize {
        self.len as usize
    }

}

