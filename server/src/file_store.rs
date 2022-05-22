use std::{collections::HashMap, io::Write};

use lunatic::process::{AbstractProcess, ProcessRef, ProcessRequest};

pub struct FileStore {
    pub files: HashMap<String, (String, Vec<u8>)>,
}

impl AbstractProcess for FileStore {
    type Arg = ();
    type State = Self;

    fn init(_: ProcessRef<Self>, _: Self::Arg) -> Self::State {
        FileStore {
            files: std::fs::read_dir("assets")
                .unwrap()
                .filter_map(Result::ok)
                .filter_map(|de| {
                    let path = de.path();
                    let filename = path.file_name()?.to_string_lossy().to_ascii_lowercase();
                    let mime_type = new_mime_guess::from_path(&path).first()?.to_string();
                    let file = {
                        let raw = std::fs::read(de.path()).unwrap();
                        let mut encoder =
                            flate2::write::GzEncoder::new(vec![], flate2::Compression::default());
                        encoder.write_all(&raw).unwrap();
                        encoder.finish().unwrap()
                    };

                    Some((filename, (mime_type, file)))
                })
                .collect(),
        }
    }
}

impl ProcessRequest<String> for FileStore {
    type Response = Option<(String, Vec<u8>)>;

    fn handle(state: &mut Self::State, path: String) -> Self::Response {
        state.files.get(&path).cloned()
    }
}
