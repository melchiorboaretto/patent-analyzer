mod metadata;
mod id_chunk;


use std::{
    fs, io::{
        ErrorKind,
        Result,
        Write
    }
};

use positioned_io::{
    RandomAccessFile, ReadAt, WriteAt
};

use metadata::{
    Metadata,
    METADATA_PAGES,
};

use id_chunk::{
    IdChunk,
    PAGE_SIZE,
};

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

    fn open_rw(&self) -> Result<RandomAccessFile>{

        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)?;

        RandomAccessFile::try_new(file)

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

        let file = self.open_rw()?;

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

    /// Appends an IdChunk into the inverted file. Returns the index.
    fn append(&mut self, file: &mut RandomAccessFile, chunk: IdChunk) -> Result<u64>{

        let append_index = self.to_append_index();

        file.write_all_at(InvertedIndex::offset(append_index), chunk.as_bytes())?;

        self.metadata.file_len += 1;

        file.write_all_at(0, self.metadata.as_bytes())?;

        Ok(append_index)

    }

    /// Insert an IdChunk into the inverted file using the best possible position.
    /// Returns the index where it was inserted
    fn insert(&mut self, file: &mut RandomAccessFile, chunk: IdChunk) -> Result<u64> {

        Ok(if self.metadata.available == 0 {

            self.append(file, chunk)?

        } else {

            todo!("HERE I WILL IMPLEMENT A BEST FIT ALGORITHM");

        })

    }

    pub fn insert_all(&mut self, index: Option<u64>, ids: Vec<u64>) -> Result<Option<u64>> {

        let mut file = self.open_rw()?;
        let first_chunk_index: Option<u64>; // Last inserted chunk
        let chunk: IdChunk;


        if let Some(index) = index {
            let first_idx_and_chunk = self.last_chunk_id_and_next(&file, index)?;
            first_chunk_index = Some(first_idx_and_chunk.0);
            chunk = first_idx_and_chunk.1;

        } else {
            first_chunk_index = None;
            chunk = IdChunk::new();
        }


        let mut chunks_iter = chunk
            .insert_all(ids)
            .into_iter()
            .rev();

        let mut first = chunks_iter.next_back().expect("THIS MUST EXIST");

        let mut next_index;

        if let Some(last_chunk) = chunks_iter.next() {
            next_index = self.insert(&mut file, last_chunk)?;

            for mut item in chunks_iter {

                item.next_is(next_index);
                next_index = self.insert(&mut file, item)?;
            }

            first.next_is(next_index);
        } else {
            first.next_is(u64::MAX); // Equivalent to -1
        }

        Ok(if let Some(real_index) = first_chunk_index {

            file.write_all_at(InvertedIndex::offset(real_index), first.as_bytes())?;
            None

        } else {

            Some(self.insert(&mut file, first)?)

        })

    }

    fn offset(index: u64) -> u64 {
        (index + METADATA_PAGES) * PAGE_SIZE
    }

    fn to_append_index(&self) -> u64 {
        self.metadata.file_len
    }

}


#[cfg(test)]
mod test {

    use super::*;
    use tempfile::*;
    use rand::*;
    use rand::seq::SliceRandom;

    #[test]
    fn create_and_manipulate_inverted_file() {
        let tempdir = tempdir().expect("UNABLE TO CREATE A TEMPORARY DIRECTORY");

        let file_path = tempdir
            .path()
            .join("testfile.idx")
            .to_string_lossy()
            .into_owned();

        let mut inv_index = InvertedIndex::new(file_path).expect("UNABLE TO CREATE THE INV. FILE");

        let mut ids = Vec::new();
        let mut rng = rng();

        for num in 0..PAGE_SIZE*9 {

            ids.push(num);

        }

        ids.shuffle(&mut rng);

        let ins_id = inv_index.insert_all(None, ids.clone()).expect("UNABLE TO INSERT THE IDS");

        let test_ids = inv_index.retrieve_ids(ins_id.unwrap()).expect("UNABLE TO RETRIEVE IDS");

        assert_eq!(ids, test_ids);

    }

}
