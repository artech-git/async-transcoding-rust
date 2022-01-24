use bytes::{Buf, BufMut};
use futures::StreamExt;
use futures::{TryStreamExt, Stream};
use std::io::IoSlice;
use std::io::Write;

use crate::obj;

pub async fn upload(
    form: warp::multipart::FormData,
    tx: tokio::sync::mpsc::UnboundedSender<obj::FileData>,
) -> Result<impl warp::Reply, warp::Rejection> {


    tracing::log::info!("inside the upload function");

    let parts: Vec<warp::multipart::Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    for p in parts {
        //if p.name() == "file" {

        let content_type = p.content_type();
        let file_ending;

        match content_type {
            Some(file_type) => match file_type {
                "application/octet-stream" => {
                    // TODO Change this ! 😁
                    file_ending = "mp4";
                }

                "video/mp4" => {
                    file_ending = "mp4";
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

        let file_name = format!(
            "./files/{}.{}",
            uuid::Uuid::new_v4().to_hyphenated().to_string(),
            file_ending
        );


        use std::cell::RefCell as RefC;

        //let mut file_ref = &file;

        /*
        let mut file = RefC::new
        ( std::io::BufWriter::new(std::fs::File::create(&file_name.clone()).unwrap()) );
        
        let file_ref = &file;

        
        p.stream().try_for_each( move |mut buf| { 
            
            async move {

                while buf.remaining() > 0 {

                    let chunk = buf.chunk();

                    file_ref
                    .borrow_mut()
                    .write_all(&chunk)
                    .map_err(|e| warp::reject::reject()); // TODO '?' file write operation

                    let t = chunk.len();

                    buf.advance(t);
                }

                Ok(())
            }


        });
                                      
        
        let mut stream = p.stream();
        
        while let Some(chunk) = stream.next().await {
            
            
        }
        */
        
        
        let value = p.stream().try_fold(
            Vec::new(), |mut vec, data|
            { vec.put(data);  async move { Ok(vec) }})
            .await
            .map_err( move |e| {eprintln!("reading file error: {}", e); warp::reject::reject() })?;
            
            
            
            tokio::fs::write(&file_name, value).await.map_err(|e| {
                eprint!("error writing file: {}", e);
                warp::reject::reject()
            })?;
            
            
            
        // construct the object type here >

        let mut file_data = obj::FileData::new();

        file_data.filePath = file_name.clone();

        tracing::log::info!(" obj: created: {:?}", file_data);

        match tx.send(file_data) {
            Ok(g) => {
                tracing::log::info!("-----send successfull-----");
            }

            Err(e) => {
                eprintln!(
                    "it failed due to some error: {:?}",
                    std::error::Error::source(&e)
                );
            }
        }

        tracing::log::info!(
            "[{}] Created file: {} \n",
            chrono::Utc::now().format("%D %H:%M:%S"),
            file_name
        );
        // }
    }

    Ok(format!("ok"))
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
