use bytes::{Buf, BufMut};
use futures::StreamExt;
use futures::{TryStreamExt, Stream};
use hyper::HeaderMap;
use hyper::header::{AUTHORIZATION, HeaderValue};
use warp::{Rejection, Filter, Reply};
use std::collections::HashMap;
use std::convert::Infallible;
use std::io::IoSlice;
use std::io::Write;

use jsonwebtoken::{self, decode, DecodingKey, Validation, Algorithm};

use crate::constants::BEARER;
use crate::defined_error::*;
use crate::obj::{self, Claims};

use tokio::process::Command;

//>> 1 >> 54 >> 23 >> 
// [4,64,1,5,4,] -> [ 1, 4, 5, 64]

//declare the supported types video content types here !
pub async fn upload(
    claim: Claims,
    form: warp::multipart::FormData,
    tx: tokio::sync::mpsc::UnboundedSender<obj::FileData>,
) -> Result<impl warp::Reply, warp::Rejection> {

    tracing::log::info!("Inside the upload function");
    //tracing::log::info!(" claim recieved: {:?}", claim);

    let parts: Vec<warp::multipart::Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    let mut resp: obj::FileData = obj::FileData::new();

    for p in parts {
        let content_type = p.content_type();
        let file_ending;

        match content_type {
            Some(file_type) => match file_type {
                
                "video/mp4" => {
                    file_ending = "mp4";
                }
                "video/webm" => {
                    file_ending = "webm"
                }
                "video/mpeg" => {
                    file_ending = "mpeg"
                }
                "video/quicktime" => {
                    file_ending = "mov"
                }
                "video/x-m4v" => {
                    file_ending = "m4v"
                }
                "video/x-msvideo" => {
                    file_ending = "avi"
                }
                "video/x-flv" => {
                    file_ending = "flv"
                }
                "video/x-mng" => {
                    file_ending = "mng"
                }
                v => {
                    eprintln!("invalid file type found: {}", v);
                    return Err(warp::reject::reject());
                }
            },
            None => {
                eprintln!("file type could not be determined");
                return Err(warp::reject::reject());
            }
        }
        let random_id = uuid::Uuid::new_v4().to_hyphenated().to_string(); 

        let file_name = format!(
            "./files/{}.{}",
            &random_id,
            file_ending
        );

        let name = p.filename().unwrap().to_string();

        let value = p.stream().try_fold(
            Vec::new(), |mut vec, data|
            
            { vec.put(data);  async move { Ok(vec) }})
            .await
            .map_err( move |e| {eprintln!("reading file error: {}", e); warp::reject::reject() })?;           
            

        tokio::fs::write(&file_name, value).await
            .map_err(|e| {
                eprint!("error writing file: {}", e);
                warp::reject::reject()
            })?;
        // construct the object type here >

                                //TODO handling of the arbitary file upload on the server while headers looks legit
        match Command::new("mediainfo")
                                    .arg(&file_name).arg("--output=JSON").output().await
                                    {
                                        Ok(out) => {

                                            if out.status.success() {                                                
                                                //let json_info = serde_json::from_str(std_out).unwrap();
                                                
                                                let json_file_path =  format!(
                                                    "./files/{}.{}",
                                                    &random_id,
                                                    "json"
                                                );
                                                
                                                tokio::fs::write(&json_file_path, &out.stdout.as_slice() )
                                                            .await
                                                            .map_err(|e| {
                                                                tracing::log::error!(" Error creating json file: [{:?}]", e);
                                                                return warp::reject::reject();
                                                            });
                                            }
                                            else {
                                                let std_err = std::str::from_utf8(&out.stderr.as_slice()).unwrap();
                                                tracing::log::warn!("Arbitary exit status issue of MediaInfo : {:?}", std_err);
                                                return Err(warp::reject::reject());
                                            }

                                        }
                                        Err(e) => {
                                            tracing::log::info!(" Error in exec of mediainfo child process : [{:?}]", e);
                                            return Err(warp::reject::reject());
                                        }
                                    }


        
        let output_hash  = match Command::new("sha256sum")
                                            .arg(&file_name)
                                            //.arg("| cut -f 1 -d ' ' ")
                                            .output().await 
                                    {
                                        Ok(out) => {
                                           
                                            //TODO fix lifetime error of out.stdout while trying to compute in match statement for &'static str type                                    

                                            if out.status.success() {
                                                String::from_utf8(out.stdout[..64].to_vec()).unwrap()           
                                            }
                                            else {
                                                let std_err = String::from_utf8(out.stderr).unwrap();
                                                tracing::log::warn!("Arbitary exit status of sha256sum : {:?}", std_err);
                                                return Err(warp::reject::reject());
                                            }
                                            

                                        },
                                        Err(e) => {
                                            tracing::log::info!(" Error in exec of Sha256sum hashing child process : [{:?}]", &e);
                                            return Err(warp::reject::reject());

                                        }
                                    };



        let mut file_data = obj::FileData::new();

        file_data.filePath = file_name.clone(); 
        file_data.uuid = random_id;
                                    
        file_data.fileName = name;
        file_data.hash = output_hash;

        //response_build.file_obj = file_data.clone();


        file_data.status(1);        // file is now scheduled 
        
       // response_build.file_obj = file_data.clone(); // clone the given object

        tracing::log::info!("FileData obj: created: {:?}", file_data);
        
         resp = file_data.clone();

        match tx.send(file_data.clone()) {
            Ok(g) => {
                tracing::log::info!(" Message Send Successfull ");            
            },
            Err(e) => {            
                eprintln!(
                    "it failed due to some error: {:?}",
                    std::error::Error::source(&e)
                );            
            }
        }

        tracing::log::info!(
            "Created file: {} \n",
           // chrono::Utc::now().format("%D %H:%M:%S"),
           &file_name
        );

    }

    Ok( resp.into_response() )
}


/*
//let known_file_types = vec!["video/mp4", "video/mkv"];

pub async fn upload(form: warp::multipart::FormData,
                    tx: tokio::sync::mpsc::UnboundedSender<obj::FileData>) -> 
        Result<impl warp::Reply , warp::Rejection>{

        use futures::stream::{self, StreamExt, Next};
        use futures::TryStreamExt;
        use bytes::BufMut;
        use tokio::io::AsyncWriteExt;

        for part in form.enumerate().next().await 
        {
            match part.1 {
                Ok(chuck) => {
                    
                    let content_type = chuck.content_type();

                    let file_ending: &str  = match content_type 
                                        {
                                                Some(file_type) => {
                                                    file_type
                                                },
                                                None => {
                                                    ""
                                                }
                                        };        


                    let file_name = 
                        format!("{}.{}", "SAMPLE", file_ending);

                    let mut file_obj = 
                            tokio::fs::File::create(file_name).await;

                    let mut file = 
                            tokio::io::BufWriter::new(file_obj.unwrap());
                    
                    let mut data_stream = chuck.stream();

                    while let Some(buffer_data) = data_stream.next().await {

                        let mut b_data = buffer_data.unwrap();






                    }               
                },
                Err(e) => {

                    //return Ok("fine".to_string());
                return Err(warp::reject::reject());
                    
                }
                
            }
        }

        Ok("success".to_string())

}
*/

pub fn user_request_body() -> 
    impl Filter<Extract = (Claims,), Error = Rejection > + Clone
{   
    tracing::log::info!("Inside req body func.");

    warp::any()
        .and(warp::header::headers_cloned())
        .and_then(authorize)
    
}


// perform the task of handling rejections in the http request route /upload
pub async fn handle_rejection(
    err: warp::Rejection,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {

    let (code, message) = if err.is_not_found() {

        (hyper::StatusCode::NOT_FOUND, "Not Found".to_string())

    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            hyper::StatusCode::BAD_REQUEST,
            "Payload too large".to_string(),
        )
    } else {
        eprintln!("unhandled error: {:?}", err);

        (
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}


fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> 
    std::result::Result<String, Error> 
{
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(Error::NoAuthHeaderError),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(Error::NoAuthHeaderError),
    };

    if !auth_header.starts_with(crate::constants::BEARER) {
        return Err(Error::InvalidAuthHeaderError);
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}


 async fn authorize( headers: HeaderMap<HeaderValue>) -> 
    //impl warp::Filter<Extract = Claims , Error = Rejection > 
    std::result::Result<Claims, Rejection>

{    
        match jwt_from_header(&headers) {
            
            Ok(jwt) => {
                
                tracing::log::info!("Decoding token");


                let decoded = 
                decode::<obj::Claims>(
                    &jwt,
                    &DecodingKey::from_secret(crate::constants::JWT_SECRET),
                    &Validation::new(Algorithm::HS512),
                )
                .map_err(|e| {
                    tracing::log::error!("failed to decode jwt : {:?}", e);
                    warp::reject::custom(Error::JWTTokenError)
                }
                )?;

                tracing::log::info!("Authorize: Ok: for JWT");
                //TODO add the error case handling the rate limit feature of user 
                Ok(decoded.claims)
       
            }
            Err(e) => 
            { 
                tracing::log::info!("JWT NOT FOUND in HEADER : Err = {:?}" , &e );
                return Err(warp::reject::custom(e));
            }
        }
}
    
pub fn file_response(id: String) -> 
    impl Filter<Extract = (warp::fs::File,), Error = Rejection > + Clone 
{
    let local_path = format!("./output/{}", id);
    warp::fs::file(local_path)
}
