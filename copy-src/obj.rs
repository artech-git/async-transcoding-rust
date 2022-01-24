use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Status {
    DONE,
    FAILED,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub filePath: String,
    pub uuid: String,
    file_type: String,
    status: Status,
    hash: String,
}

impl FileData {
    pub fn new() -> Self {
        Self {
            filePath: "nil".to_string(),
            uuid: "nil".to_string(),
            file_type: "nil".to_string(),
            hash: "nil".to_string(),
            status: Status::DONE,
        }
    }
}
