
const PAGE_SIZE: u64 = 512;
const METADATA_PAGES: u64 = 1;
const METADATA_FIELDS: u64 = 3;
const MAX_IDS_PER_CHUNK: u64 = ((PAGE_SIZE as usize - (size_of::<u64>() + size_of::<i64>())) / size_of::<u64>()) as u64;


use std::{
    fs, 
    io::{
        ErrorKind,
        Result,
        Write
    }
};

use positioned_io::{
    RandomAccessFile, ReadAt, WriteAt
};

#[repr(C)]
struct Metadata {
    file_len: u64,
    recycled: u64,
    available: u64, // If available == 0, the idea is to auto-append, instead of applying some
    // best-fit algorithm
    free_idx: [u64; (PAGE_SIZE/8 - METADATA_FIELDS) as usize],
}

impl Metadata {

    fn new() -> Self {
        Metadata {
            file_len: 0,
            recycled: 0,
            available: 0,
            free_idx: [0; (PAGE_SIZE/8 - METADATA_FIELDS) as usize]
        }
    }

    fn as_bytes(&self) -> &[u8] {

        unsafe {
            let raw_ptr = self as *const Metadata as *const u8; 

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

}


/// The first PAGE_SIZE of the file will store, in order, how many indexes are being used (len), how many
/// of these list were used in the past (recycled), how many
/// indexes were deleted (if it is 0, the next field has garbage. Available field), all the free indexes that must be
/// read before doing any operations (free_idx).
///
/// IMPORTANT: The "auto-vacuum" criteria will be smth like 80% of the free indexes list occupied
/// or every 2^n pages (n > 10)
pub struct InvertedIndex {
    path: String,
    metadata: Metadata,
}

impl InvertedIndex {

    /// Initializes the InvertedIndex using a known file path. If the file does not exist
    /// it will be created, if it does, nothing more happens.
    pub fn new(path: String) -> Result<InvertedIndex> {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path) {
            Ok(mut new_file) => {

                let metadata = Metadata::new();
                new_file.write_all(metadata.as_bytes())?;

                Ok(InvertedIndex {
                    path,
                    metadata,
                })
            },

            Err(error) => {

                if error.kind() == ErrorKind::AlreadyExists {

                    let file =  fs::OpenOptions::new()
                        .read(true)
                        .open(&path)?;

                    let mut bytes_buffer = [0u8; PAGE_SIZE as usize];
                    file.read_exact_at(0, &mut bytes_buffer)?;

                    let metadata = unsafe {
                        Metadata::from_bytes(&bytes_buffer)
                    };

                    Ok(InvertedIndex {
                        path,
                        metadata,
                    })

                } else {
                    Err(error)
                }

            },
        }
    }

    // Get the necessary data for adding more IDs, the last chunk index and itself
    fn last_chunk_id_and_next(&self, file: &RandomAccessFile, mut index: u64) -> Result<(u64, IdChunk)> {

        let mut chunk = IdChunk::read_new(file, index)?;

        loop {
            if chunk.next == -1 {
                break Ok((index, chunk));
            }

            index = chunk.next();
            chunk = IdChunk::read_new(file, index)?;

        }

    }


    /// Aux function to read a block of the inverted index, appends it to a vector.
    /// It is better if the vector is already initialized with the right size to avoid
    /// syscalls asking more memory
    fn read_chunk_to_vec(&self, ids: &mut Vec<u64>, file: &RandomAccessFile, index: u64) -> Result<Option<u64>> {

        let chunk = IdChunk::read_new(file, index)?;

        ids.extend_from_slice(&chunk.content[0..chunk.len()]);

        Ok(if chunk.next != -1 {
            Some(chunk.next as u64)
        } else {
            None
        })
    }

    /// Retrieves all the ids for a given index
    pub fn retrieve_ids(&self, mut index: u64) -> Result<Vec<u64>> {
        // NOTE: THIS FUNCTION MAY NOT BE USED TO INSERT AN ID / UPDATE A BLOCK.
        // ITS WISE (IF AN UPDATE IS WANTED), TO READ THE BLOCK AND SAVE ITS "CREDENTIALS",
        // read_block_to_vec() does something alike. 

        let file = RandomAccessFile::open(&self.path)?;

        let mut ids_vector = Vec::new();

        loop {

            let next = self.read_chunk_to_vec(&mut ids_vector, &file, index)?;

            if let Some(next_offset) = next {
                index = next_offset;
            } else {
                break;
            }
        }

        Ok(ids_vector)
    }

    fn append(&mut self, file: &mut RandomAccessFile, chunk: IdChunk) -> Result<()>{

        file.write_all_at(self.to_append_offset(), chunk.as_bytes())?;

        self.metadata.file_len += 1;

        file.write_all_at(0, self.metadata.as_bytes())?;

        Ok(())

    }

    fn insert(&mut self, file: &mut RandomAccessFile, chunk: IdChunk) -> Result<()> {

        if self.metadata.available == 0 {

            self.append(file, chunk)?;

        } else {

            todo!("HERE I WILL IMPLEMENT A BEST FIT ALGORITHM");

        }

        Ok(())

    }

    fn offset(index: u64) -> u64 {
        (index + METADATA_PAGES) * PAGE_SIZE
    }

    fn to_append_offset(&self) -> u64 {
        InvertedIndex::offset(self.metadata.file_len)
    }

}

#[repr(C)]
struct IdChunk {
    len: u64, // Here I used u64 for padding reasons.
    next: i64, // If next == -1, there's no next
    content: [u64; MAX_IDS_PER_CHUNK as usize],
}

impl IdChunk {

    /// Creates a blank IdChunk
    fn new() -> Self {
        IdChunk {
            len: 0,
            next: -1,
            content: [0; MAX_IDS_PER_CHUNK as usize],
        }
    }

    fn read_new(file: &RandomAccessFile, index: u64) -> Result<Self> {
        unsafe {
            let mut bytes = [0; PAGE_SIZE as usize];

            file.read_exact_at(InvertedIndex::offset(index), &mut bytes)?;
            Ok(IdChunk::from_bytes(&bytes))
        }
    }

    /// Tries to insert the given ID into the chunk, returns true if it could.
    /// Else, returns false.
    /// NOTE: It WON'T adjust any next pointers, this must be done manually.
    fn try_insert(&mut self, id: u64) -> bool {
        if self.len == MAX_IDS_PER_CHUNK {
            false
        } else {
            self.content[self.len()] = id;
            self.len += 1;
            true
        }

    }

    // adjust the next field of a chunk. 
    fn next_is(&mut self, index: u64) {
        self.next = index as i64;
    }

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

    fn next(&self) -> u64 {
        self.next as u64
    }

}

