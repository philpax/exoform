use std::{collections::HashMap, io::Write, path::PathBuf};

use lunatic::process::{AbstractProcess, ProcessRef, ProcessRequest};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct File {
    pub mime_type: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}
impl File {
    pub fn new(mime_type: String, data: Vec<u8>) -> Self {
        Self { mime_type, data }
    }
}

pub struct FileStore {
    pub files: HashMap<String, (PathBuf, String)>,
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

                    Some((filename, (path, mime_type)))
                })
                .collect(),
        }
    }
}

impl ProcessRequest<String> for FileStore {
    type Response = Option<File>;

    fn handle(state: &mut Self::State, path: String) -> Self::Response {
        let (path, mime_type) = state.files.get(&path)?;
        let file = {
            let raw = std::fs::read(path).unwrap();
            let mut encoder = flate2::write::GzEncoder::new(vec![], flate2::Compression::default());
            encoder.write_all(&raw).unwrap();
            encoder.finish().unwrap()
        };

        Some(File::new(mime_type.clone(), file))
    }
}
