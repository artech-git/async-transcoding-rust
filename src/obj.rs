use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::{Filter, http::response};

use http::{ header::{HeaderValue,  ACCESS_CONTROL_ALLOW_ORIGIN, SERVER, CACHE_CONTROL} , HeaderMap, StatusCode};
//use hyper::Response;



#[derive(Clone, Debug, Serialize)]
pub enum Status {
    SCHEDULED,
    PROCESSING,
    DONE,
    FAILED,
}

//==========================================================================
#[derive(Debug, Clone, Serialize)]
pub struct FileData {

    pub filePath: String,
    pub uuid: String,
    pub file_type: String,
    pub hash: String,
    pub dir : String,
    pub fileName: String,
    pub download_route: String,
    pub status: Status,
    
}

impl FileData {
    pub fn new() -> Self {
        Self {
            filePath: "nil".to_string(),
            uuid: "nil".to_string(),
            file_type: "nil".to_string(),   
            hash: "nil".to_string(),
            status: Status::DONE,
            dir: "nil".to_string(),
            fileName: "nil".to_string(),
            download_route: "nil".to_string(),
        }
    }
    pub fn status(&mut self, s: u8) {
        match s {
            1 => self.status = Status::SCHEDULED,
            2 => self.status = Status::PROCESSING,
            3 => self.status = Status::DONE,
            4 => self.status = Status::FAILED,
            _ => self.status = Status::FAILED
        };
    }
}

impl warp::Reply for FileData {

    fn into_response(self) -> warp::reply::Response {

        let json_return_type = json!({
            "success": true,
            "fileId": &self.uuid,
            "fileName": &self.fileName,
            "downloading_route": &self.download_route,
            "hash": &self.hash,
            "status": &self.status
        });
    
        let mut resp = warp::reply::json(&json_return_type).into_response();
        
        // resp.status_mut() = StatusCode::CREATED;
        //todo fix the runtime panic issue with the header configuration 
        let _ = resp.headers_mut().insert(SERVER, HeaderValue::from_static("Rbucket")).unwrap();
        //let _ = resp.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*")).unwrap();
        //let _ = resp.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("no-cache")).unwrap();            
            
            return resp;
    }
}

//==========================================================================
#[derive(Clone , Debug, Deserialize, Serialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

//==========================================================================
#[derive(Clone, Debug, Serialize)]
pub struct UploadFileResponse {
    pub fileId : String,
    pub fileName: String,
    pub downloading_route: String,
    pub file_obj: FileData, //TODO add explicit lifetime qualifier handler here for representing the fileData types inside it 
}

impl UploadFileResponse {
    pub fn new() -> Self {

        Self {
            fileId: "nil".to_string(),
            fileName: "nil".to_string(),
            downloading_route: "nil".to_string(),
            file_obj : FileData::new()
        }
    }
  
}

impl warp::Reply for UploadFileResponse {

fn into_response(self) -> warp::reply::Response {
 
    //header_map.insert()

    let json_return_type = json!({
        "success": true,
        "fileId": &self.fileId,
        "fileName": &self.fileName,
        "downloading_route": &self.downloading_route,
        "hash": &self.file_obj.hash,
        "status": &self.file_obj.status
    });

    let mut  resp = warp::reply::json(&json_return_type.to_string()).into_response();
    
    *resp.status_mut() = StatusCode::CREATED;

    resp.headers_mut().insert(SERVER, HeaderValue::from_static("Rbucket")).unwrap();
    resp.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*")).unwrap();
    resp.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("no-cache")).unwrap();            
        
        return resp;
    }


}