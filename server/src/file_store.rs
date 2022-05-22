use std::collections::HashMap;

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
                    Some((
                        path.file_name()?.to_string_lossy().to_ascii_lowercase(),
                        (
                            new_mime_guess::from_path(&path).first()?.to_string(),
                            std::fs::read(de.path()).unwrap(),
                        ),
                    ))
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
