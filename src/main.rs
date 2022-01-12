#[allow(dead_code, unused_imports)]
use std::convert::Infallible;
use std::error::Error;
use std::sync::Arc;

use chrono::Utc;
//use hyper::body::HttpBody;
use hyper::{Server, StatusCode};
use hyper::service::make_service_fn;

#[allow(unused_imports)]

use log::{info, warn, error};

use bytes::{self, Bytes};
use bytes::Buf;

use serde::{Serialize, Deserialize};
use tokio::sync::{self, mpsc, oneshot, watch};
use tokio::task;
//use tokio::io::{AsyncBufReadExt,  AsyncWriteExt, AsyncReadExt, self,};

use std::process::Command;
use warp::{Reply, Filter, Rejection};
use warp::multipart::{FormData, Part};

//use uuid::Uuid;

use bytes::BufMut;
use futures::TryStreamExt;


#[derive(Debug,Clone, Serialize, Deserialize)]
struct FileData {
    pub filePath : String,
    pub uuid: String,

}


/*
async fn attempt_read_file(mut rx: mpsc::UnboundedReceiver<String>) -> 
        Result<(), Box<dyn std::error::Error>>
{
    Ok(())
}*/



fn with_tx(tx: mpsc::UnboundedSender<String>) -> 
    impl Filter<Extract = (mpsc::UnboundedSender<String>,), Error = Infallible> + Clone 
{
    warp::any().map(move || tx.clone())
}


//extern crate console_subscriber;


#[tokio::main]
async fn main() {
    //console_subscriber::init();

    let port: u16 = env!("SRV_PORT").parse::<u16>().unwrap();
    println!("server started on port : {:?}", &port);

    //let buffer_service  = Arc::new(Vec::new());
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let (child_tx, mut child_rx) = watch::channel::<String>("init".to_string());
    //------------------------------------------------------------------------------------------------
    //attempt_read_file(rx);

    task::spawn_blocking( move ||  {

                while let Some(filePath) = rx.blocking_recv() 
                {   
                    println!("[{:?}] Attempting to read file: {:?}",Utc::now(), &filePath);

                    let file_name = 
                    format!("./output/{}.{}", uuid::Uuid::new_v4().to_hyphenated().to_string(), ".avi");
                    
                    child_tx.send(file_name.clone()).unwrap();

                    let ffmpeg_process = 
                                    if cfg!(target_os = "windows"){
                                            // Windows 
                                                Command::new("powershell")
                                                    .arg("ffmpeg")
                                                    .args([ "-i", &filePath, "-r",  "24",  &file_name ])
                                                    .output()
                                    }
                                    else {  //switch to the linux context for the
                                                Command::new("sh")
                                                .args(["ffmpeg -h"])
                                                .output()
                                    };
                    
                    //send the status of child process
                    //let status = ffmpeg.

                    let out = ffmpeg_process.expect("ERROR!").stdout;

                    println!("ouput of child process :{:?}", String::from_utf8(out));      
                    child_tx.send("NULL".to_string()).unwrap();                 
                    }
                }
        );


        let core_route = warp::path("upload");
    //------------------------------------------------------------------------------------------------
        let upload = core_route
                                        .and(warp::get())
                                        .and(warp::multipart::form().max_length(50000000000))
                                        .and(with_tx(tx.clone()))
                                        .and_then(upload);
                                        //.with(warp::log("completed the upload"));

        
        let check_queue = core_route
        .and(warp::path("check"))
        .and(warp::get())
        //.and(with_rx(child_rx.clone()))
        .map(move  || {
            let stat = child_rx.borrow().clone();
            format!("This process is in exec: {:?}", stat)
        });
        
    

    let def_route = warp::service(upload
                                                                    .or(check_queue)
                                                                    //.recover(handle_rejection)
                                                            );

    //------------------------------------------------------------------------------------------------
    

        let def_route_gen = def_route.clone();

        let make_svc = make_service_fn(move |_| {

        let def_route = def_route_gen.clone();

            async move {
                info!("Hyper interior service started");
                println!("");
                println!("---Inside def_route handler-----");
                Ok::<_, Infallible>(def_route)  //return the value generated 
            }
        });


        let server = 
                                    Server::bind(&([0,0,0,0], port.clone()).into())
                                            //.http2_only(true)
                                            .serve(make_svc);
        
                                    
        if let Err(e) = server.await {
            eprintln!("server error: {}", &e);
            error!("server falid to continue: {:?}", &e);
        }
        

    
}


async fn upload(form: FormData ,tx : mpsc::UnboundedSender<String>) ->
        Result<impl Reply, warp::Rejection>
{

    let parts: Vec<Part> = 
            form.try_collect().await.map_err(|e| {
                    eprintln!("form error: {}", e);
                    warp::reject::reject()
                })?;

                println!("inside the upload function");
                
    for p in parts {
        //if p.name() == "file" {

            let content_type = p.content_type();
            let file_ending;

            match content_type {

                Some(file_type) => match file_type {
                    "application/pdf" => {
                        file_ending = "pdf";
                    },
                
                    "video/mp4" => {
                        file_ending = "mp4";
                    },
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

            let value = p.stream().try_fold(Vec::new(), |mut vec, data| {
                    vec.put(data);
                    async move { Ok(vec) }
                })
                .await
                .map_err(|e| {
                    eprintln!("reading file error: {}", e);
                    warp::reject::reject()
                })?;

            let file_name = format!("./files/{}.{}", uuid::Uuid::new_v4().to_hyphenated().to_string(), file_ending);
            
            tokio::fs::write(&file_name, value).await.map_err(|e| {
                eprint!("error writing file: {}", e);
                warp::reject::reject()
            })?;

            match tx.send(file_name.clone()) {
                
                Ok(g) => { println!("\n-----send successfull-----");}

                Err(e) => { eprintln!("it failed due to some error: {:?}", e.source()); }
             }
             
            println!("[{:?}] Created file: {} \n",Utc::now(), file_name);
        //}
    }

    Ok("success")

}


async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {

    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}