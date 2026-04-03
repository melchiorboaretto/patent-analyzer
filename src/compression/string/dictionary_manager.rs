
use crate::compression::string::{

    FOLDERNAME,
    HEADER_FILENAME,
    DICTIONARY_FILENAME,

    dictionary::{
        Dictionary,
    },
};

use std::{

    collections::HashMap,

    io::Result,

    path::PathBuf,

    sync::Arc,

};

pub struct DictionaryManager {
    path: String,
    dict_cache: HashMap<u64, Arc<Dictionary>>,
}

impl<'a> DictionaryManager {

    #[cfg(test)]
    fn __test_set_path(&mut self, new_path: String) {

        self.path = new_path;

    }

    pub fn new() -> Self {

        DictionaryManager {
            path: String::from(FOLDERNAME),
            dict_cache: HashMap::new(),
        }

    }

    pub fn populate<C: IntoIterator<Item = &'a str>>(&mut self, id: u64, data: C) -> Result<bool>{

        let folder_path = PathBuf::from(&self.path);

        let test_offset_size = Dictionary::get_offset_size(
            folder_path.join(HEADER_FILENAME),
            id
        );

        let test_offset_size = match test_offset_size {

            Err(error) => {

                if error.kind() == std::io::ErrorKind::NotFound {

                    std::fs::create_dir_all(&self.path)?;

                    Dictionary::create_files(self.path.clone())?;

                } else {
                    return Err(error)
                }

                None

            }

            Ok(offset_size) => offset_size,

        };

        // Guard to test if the dictionary was already populated
        if test_offset_size.is_some() {
            return Ok(false);
        }

        // If it was not, populate it and save the metadata
        let new_dict = Dictionary::from_strings(data);

        let offset_size = new_dict.export_to_file(folder_path.join(DICTIONARY_FILENAME))?;
        Dictionary::set_offset_size(folder_path.join(HEADER_FILENAME), offset_size, id)?;

        Ok(true)

    }

    pub fn get(&mut self, id: u64) -> Result<Option<Arc<Dictionary>>> {

        // Tests for cache
        if let Some(cached_dict) = self.dict_cache.get(&id) {

            return Ok(Some(cached_dict.clone()));

        }

        // Cache miss
        let folder_path = PathBuf::from(&self.path);

        let dict = Dictionary::from_file_std(folder_path, id)?;

        if let Some(dict) = dict {
            let arc_dict = Arc::new(dict);
            self.dict_cache.insert(id, arc_dict.clone());
            Ok(Some(arc_dict))
        } else {
            Ok(None)
        }

    }


}

#[cfg(test)]
mod test {

    use tempfile::*;
    use crate::compression::string::{
        codec::*,
        dictionary_manager::*,
    };

//    #[test]
//    fn compress_and_decompress() {
//
//        let test_string = "Gaúcha Zero Hora 28/01/2026 - 16:50h Anvisa aprova cultivo de cannabis para fins medicinais.\
//            De acordo com o texto, a produção de cannabis só será autorizada para fins medicinais e farmacêuticos, \
//            sendo restrita a pessoas jurídicas. Os estabelecimentos só poderão produzir a quantidade necessária para atender a uma demanda \
//            de medicamentos autorizada previamente. Ainda conforme a proposta, o teor de THC deverá ser no máximo de 0,3%. As áreas de cultivo \
//            serão limitadas, devendo ser georreferenciadas, fotografadas e monitoradas. Segundo a Anvisa, tratam-se de áreas pequenas, que serão \
//            acompanhadas de perto pela agência. Para o transporte dos produtos, a Anvisa informou que será \
//            firmada uma parceria com a Polícia Rodoviária Federal. \
//            \
//            Fila impactante com quase três quilômetros de vagões parados é retrato da decadência das ferrovias no RS; assista \
//            Governos e empresas buscam alternativas para conter a precarização da malha ferroviária do Estado, que perdeu 75% do \
//            tamanho em três décadas. \
//            \
//            Ficou mais fácil para os bandidos, diz Armínio Fraga sobre infiltração no sistema financeiro \
//            Ex-presidente do Banco Central diz que o mundo descoordenado ajudou atividades ilegais e vê como \
//            maior erro no caso Master o descuido com uso de fundos no balanço do banco. \
//            \
//            Sucesso nas redes sociais, chocolate Trento chega ao menu de sorvetes do McDonald's \
//            Sobremesa McFlurry Trento Bites entra oficialmente nas lojas de todo o Brasil em 2 de setembro \
//            \
//            In this example, the spawned thread is “detached,” which means that there is no way for the program \
//            to learn when the spawned thread completes or otherwise terminates. \
//            To learn when a thread completes, it is necessary to capture the JoinHandle object that is \
//            returned by the call to spawn, which provides a join method that allows the caller to wait for the \
//            completion of the spawned thread: \
//            use std::thread; \
//            let thread_join_handle = thread::spawn(move || { \
//            // some work here \
//            }); \
//            // some work here \
//            let res = thread_join_handle.join(); \
//            The join method returns a thread::Result containing Ok of the final value produced by the \
//            spawned thread, or Err of the value given to a call to panic! if the thread panicked. \
//            Note that there is no parent/child relationship between a thread that spawns a new thread and \
//            the thread being spawned. In particular, the spawned thread may or may not outlive the spawning \
//            thread, unless the spawning thread is the main thread.";
//
//        let dict_not_optimal_words = ["de", "a", "o", "que", "e", "do", "da", "em", "um", "para", "é",
//            "com", "não", "uma", "os", "no", "se", "na", "por", "mais", "as", "dos", "como", "mas", "foi",
//            "ao", "ele", "das", "tem", "à", "seu", "sua", "ou", "ser", "quando", "muito", "nos", "já", "está",
//            "eu", "também", "só", "pelo", "pela", "até", "isso", "ela", "entre", "depois", "sem", "mesmo", "aos",
//            "ter", "seus", "quem", "nas", "me", "esse", "eles", "estão", "você", "tinha", "foram", "essa", "num",
//            "nem", "suas", "meu", "às", "minha", "têm", "numa", "pelos", "elas", "havia", "seja", "qual", "era",
//            "fazer", "dois", "toda", "outro", "te", "comigo", "fui", "foi", "estou", "agora", "pois", "deve", "do",
//            "diz", "está", "toda", "nossa", "pode", "tão", "alguns", "onde", "aqui", "será", "vida", "antes", "ano",
//            "casa", "dia", "homem", "moço", "senhor", "palavra", "filho", "noite", "amigo", "bem", "rua", "vida", "hora",
//            "coração", "pai", "pessoa", "mulher", "amor", "verdade", "ideia", "mãe", "marido", "espírito", "fim"];
//
//        let dict = Dictionary {
//            entries: dict_not_optimal_words.iter().map(|str| str.to_string()).collect(),
//            lookup_map: OnceLock::new(),
//        };
//
//        let overkill_dict = Dictionary::from_strings(vec![test_string]);
//        let other_overkill_dict = Dictionary {
//            entries: overkill_dict.entries.clone(),
//            lookup_map: OnceLock::new(),
//        };
//
//        let overcompressed = CompressedString::compress(test_string, Arc::new(overkill_dict));
//
//        let compressed = CompressedString::compress(test_string, Arc::new(dict));
//
//        assert_eq!(compressed.decompress(), test_string);
//        assert_eq!(compressed.decompress(), overcompressed.decompress());
//
//        assert_eq!(format!("{}", compressed), format!("{}", test_string));
//
//        // I'm testing the file handling below
//        let tempdir = tempdir().expect("UNABLE TO CREATE A TEMPORARY DIRECTORY");
//        let file_path = tempdir
//            .path()
//            .to_owned();
//
//        let first_id = 42;
//        let second_id = 43;
//
//        Dictionary::create_files(&file_path).unwrap();
//        let offset_size = overcompressed.dict.export_to_file(file_path.join(DICTIONARY_FILENAME)).unwrap();
//        Dictionary::set_offset_size(file_path.join(HEADER_FILENAME), offset_size, first_id).unwrap();
//
//        let offset_size = compressed.dict.export_to_file(file_path.join(DICTIONARY_FILENAME)).unwrap();
//        Dictionary::set_offset_size(file_path.join(HEADER_FILENAME), offset_size, second_id).unwrap();
//
//        let neo_overkill_dict = Dictionary::from_file_std(&file_path, 42).unwrap().unwrap();
//
//        for index in 0..other_overkill_dict.entries.len() {
//            assert_eq!(other_overkill_dict.entries[index], neo_overkill_dict.entries[index]);
//        }
//
//
//    }

    #[test]
    fn client_interface() {

    let test_string = "Gaúcha Zero Hora 28/01/2026 - 16:50h Anvisa aprova cultivo de cannabis para fins medicinais.\
        De acordo com o texto, a produção de cannabis só será autorizada para fins medicinais e farmacêuticos, \
        sendo restrita a pessoas jurídicas. Os estabelecimentos só poderão produzir a quantidade necessária para atender a uma demanda \
        de medicamentos autorizada previamente. Ainda conforme a proposta, o teor de THC deverá ser no máximo de 0,3%. As áreas de cultivo \
        serão limitadas, devendo ser georreferenciadas, fotografadas e monitoradas. Segundo a Anvisa, tratam-se de áreas pequenas, que serão \
        acompanhadas de perto pela agência. Para o transporte dos produtos, a Anvisa informou que será \
        firmada uma parceria com a Polícia Rodoviária Federal. \
        \
        Fila impactante com quase três quilômetros de vagões parados é retrato da decadência das ferrovias no RS; assista \
        Governos e empresas buscam alternativas para conter a precarização da malha ferroviária do Estado, que perdeu 75% do \
        tamanho em três décadas. \
        \
        Ficou mais fácil para os bandidos, diz Armínio Fraga sobre infiltração no sistema financeiro \
        Ex-presidente do Banco Central diz que o mundo descoordenado ajudou atividades ilegais e vê como \
        maior erro no caso Master o descuido com uso de fundos no balanço do banco. \
        \
        Sucesso nas redes sociais, chocolate Trento chega ao menu de sorvetes do McDonald's \
        Sobremesa McFlurry Trento Bites entra oficialmente nas lojas de todo o Brasil em 2 de setembro \
        \
        In this example, the spawned thread is “detached,” which means that there is no way for the program \
        to learn when the spawned thread completes or otherwise terminates. \
        To learn when a thread completes, it is necessary to capture the JoinHandle object that is \
        returned by the call to spawn, which provides a join method that allows the caller to wait for the \
        completion of the spawned thread: \
        use std::thread; \
        let thread_join_handle = thread::spawn(move || { \
        // some work here \
        }); \
        // some work here \
        let res = thread_join_handle.join(); \
        The join method returns a thread::Result containing Ok of the final value produced by the \
        spawned thread, or Err of the value given to a call to panic! if the thread panicked. \
        Note that there is no parent/child relationship between a thread that spawns a new thread and \
        the thread being spawned. In particular, the spawned thread may or may not outlive the spawning \
        thread, unless the spawning thread is the main thread.";


        let tempdir = tempdir().expect("UNABLE TO CREATE A TEMPORARY DIRECTORY");
        let file_path = tempdir
            .path()
            .to_string_lossy();

        let path_string = String::from(file_path);

        // Here starts the test 
        let mut dict_mgr = DictionaryManager::new();

        dict_mgr.__test_set_path(path_string);

        dict_mgr.populate(42, vec![test_string]).expect("IT WAS NOT POSSIBLE TO POPULATE THE DICTIONARY!");

        let my_dict = dict_mgr.get(42).unwrap().unwrap();

        let compressed = CompressedString::compress(test_string, my_dict.clone());

        assert_eq!(format!("{}", compressed), format!("{}", test_string));

        // Cache test
        dict_mgr.populate(93, vec!["This is a test driven text, compression does not matter"]).unwrap();

        let first_ptr = dict_mgr.get(93).unwrap().unwrap();
        let second_ptr = dict_mgr.get(93).unwrap().unwrap();

        assert!(std::sync::Arc::ptr_eq(&first_ptr, &second_ptr));
    }
}
