








#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct FileData {
    pub filePath : String,
    pub uuid: String,
    status: enum{ DONE, FAILED},
    type: String,

}

impl FileData {

    fn new(&self) -> Self {
        Self { filePath: "nil".to_string(), 
                uuid:    "nil".to_string(), 
                status:     DONE, 
                type:       "none".to_string()
            }
    }

    

}