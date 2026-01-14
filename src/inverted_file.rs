
const PAGE_SIZE: u64 = 512;
const MAX_IDS_PER_CHUNK: u64 = ((PAGE_SIZE as usize - (size_of::<u64>() + size_of::<i64>())) / size_of::<u64>()) as u64;

use positioned_io::{
    ReadAt,
    RandomAccessFile,
};

struct InvertedIndex {
    path: String,
}

impl InvertedIndex {

    fn read_block_to_vec(&self, ids: &mut Vec<u64>, file: &RandomAccessFile, index: u64) -> Option<u64> {

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

    fn get_ids(&self, mut index: u64) -> Vec<u64> {
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

    fn as_bytes(&self) -> &[u8] {

        unsafe {
            let raw_ptr = self as *const IdChunk as *const u8; 

            std::slice::from_raw_parts(raw_ptr, size_of::<Self>())
        }

    }

    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let raw_ptr = bytes.as_ptr() as *const Self;

        unsafe {
            std::ptr::read_unaligned(raw_ptr)
        }
    }

    fn len(&self) -> usize {
        self.len as usize
    }

}

