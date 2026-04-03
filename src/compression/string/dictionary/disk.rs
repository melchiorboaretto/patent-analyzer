
use crate::compression::string::{

    HEADER_FILENAME,
    DICTIONARY_FILENAME,
    OFFSET_SIZE_IN_BYTES,

    dictionary::{
        Dictionary,
    }

};

use std::{

    io::{
        Result,
        Seek,
        Write,
    },

    path::Path,

    sync::OnceLock,

};

use positioned_io::{
    ReadAt,
    WriteAt,
};

// Implements Disk I/O
impl Dictionary {

    /// Create a pair header/dictionaries of files.
    /// Because the files do not contain any kind of code in the beggining
    /// or something alike, at least not yet; this function simply creates two new files
    pub fn create_files(folder_path: impl AsRef<Path>) -> Result<()> {

        let folder_path = folder_path.as_ref();

        std::fs::OpenOptions::new()
            .truncate(false)
            .create(true)
            .write(true)
            .open(folder_path.join(HEADER_FILENAME))?;

        std::fs::OpenOptions::new()
            .truncate(false)
            .create(true)
            .write(true)
            .open(folder_path.join(DICTIONARY_FILENAME))?;

        Ok(())
    }

    pub fn export_to_file(&self, dictionaries_path: impl AsRef<Path>) -> Result<(u64, u64)> {

        let mut block_size = 0;

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(dictionaries_path)?;

        // TA AQUI EU PRECISO DESCOBRIR O OFFSET CERTO DO ARQUIVO EM QUE VAI SER POSTO O PRIMEIRO
        // NEGOCIO,
        let offset = file.seek(std::io::SeekFrom::End(0))?;

        for word in &self.entries {
            file.write_all(word.as_bytes())?;
            file.write_all(&[0u8])?;
            block_size += word.len() + 1;
        }

        Ok((offset, block_size as u64))

    }

    pub fn set_offset_size(header_path: impl AsRef<Path>, offset_size: (u64, u64), id: u64) -> Result<()> {

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(header_path)?;

        let offset = offset_size.0.to_le_bytes();
        let size = offset_size.1.to_le_bytes();

        let offset_size = [offset, size];

        file.write_all_at(OFFSET_SIZE_IN_BYTES * id, &offset_size.concat())

    }

    /// If the size (return.1) is 0, implies that the dictionary does not exist. And MUST be
    /// updated to be used
    ///
    /// Reminder: If the Return value is Err and the file does not exist, it must be created
    /// OUT of this function. Moreover, the files exist in pairs (header/dictionaries)
    pub fn get_offset_size(header_path: impl AsRef<Path>, id: u64) -> Result<Option<(u64, u64)>> {

        let file_open = std::fs::File::open(header_path);

        match file_open {

            Err(error) => {

                Err(error)

            },

            Ok(file) => {

                let mut raw_bytes = [0; OFFSET_SIZE_IN_BYTES as usize];

                match file.read_exact_at(id * OFFSET_SIZE_IN_BYTES, &mut raw_bytes) {

                    Ok(_) => {
                        let (offset_bytes, size_bytes) = raw_bytes.split_at(size_of::<u64>());

                        let offset = u64::from_le_bytes(offset_bytes.try_into().unwrap());
                        let size = u64::from_le_bytes(size_bytes.try_into().unwrap());

                        Ok(if size == 0 {
                            None

                        } else {
                            Some((offset, size))
                        })
                    },

                    Err(error) => {

                        if error.kind() == std::io::ErrorKind::UnexpectedEof {

                            Ok(None)

                        } else {

                            Err(error)

                        }

                    }
                }

            },
        }
    }

    fn words_from_offset_size(dictionaries_path: impl AsRef<Path>, offset_size: (u64, u64)) -> Result<Vec<String>> {

        let dictionaries = std::fs::File::open(dictionaries_path)?;

        let mut raw_bytes = vec![0u8; offset_size.1 as usize];
        dictionaries.read_exact_at(offset_size.0, &mut raw_bytes)?;

        let mut return_vec: Vec<String> = raw_bytes
            .split(|b| *b == 0)
            .map(|slice| String::from_utf8(slice.to_vec()).unwrap())
            .collect();

        return_vec.pop();

        Ok(return_vec)

    }

    fn from_file(header_path: impl AsRef<Path>, dictionaries_path: impl AsRef<Path>, id: u64) -> Result<Option<Dictionary>> {

        if let Some(offset_size) = Dictionary::get_offset_size(header_path, id)? {

            let words = Dictionary::words_from_offset_size(dictionaries_path, offset_size)?;

            Ok(
                Some(
                    Dictionary {
                        entries: words,
                        lookup_map: OnceLock::new(),
                    }
                )
            )


        } else {

            Ok(None)
        }
    }

    /// Uses the standard header and dictionaries file names.
    pub fn from_file_std(folder_path: impl AsRef<Path>, id: u64) -> Result<Option<Dictionary>> {
        let path = folder_path.as_ref();

        Dictionary::from_file(path.join(HEADER_FILENAME), path.join(DICTIONARY_FILENAME), id)
    }

}
