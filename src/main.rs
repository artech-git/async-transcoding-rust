mod handler;
mod obj;
mod constants;
mod defined_error;

#[allow(dead_code, unused_imports)]
use std::convert::Infallible;
use std::error::Error;
use std::sync::Arc;

use chrono::Utc;
//use hyper::body::HttpBody;
use hyper::service::make_service_fn;
use hyper::{Server, StatusCode};

use tracing::error;
#[allow(unused_imports)]
use tracing::{event, field, info, span, Level};
use tracing_subscriber;

use bytes::Buf;
use bytes::{self, Bytes};

use serde::{Deserialize, Serialize};
use tokio::sync::{self, mpsc, oneshot, watch};
use tokio::task;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
//use tokio::io::{AsyncBufReadExt,  AsyncWriteExt, AsyncReadExt, self,};

use std::process::Command;
use warp::multipart::{FormData, Part};
use warp::{Filter, Rejection, Reply};

use crate::obj::FileData;

//use uuid::Uuid;

/*
async fn attempt_read_file(mut rx: mpsc::UnboundedReceiver<String>) ->
        Result<(), Box<dyn std::error::Error>>
{
    Ok(())
}*/

fn with_tx(
    tx: mpsc::UnboundedSender<obj::FileData>,
) -> impl Filter<Extract = (mpsc::UnboundedSender<obj::FileData>,), Error = Infallible> + Clone {

    warp::any().map(move || tx.clone())
}
/*
fn rate_limit() ->
    impl Filter<

*/


//enable console subsscriber .. method in call 
//extern crate console_subscriber;
extern crate redis;
extern crate couchdb;

#[tokio::main]
async fn main() {

    //console_subscriber::init();
    
    /*                              WE HAVE TO WORK ON THIS
    let timer = LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_timer(timer));
        //.with(EnvFilter::from_default_env())
        //.with_max_level(Level::TRACE);
    */
    tracing_subscriber::fmt::init();

    let port: u16 = (3000u16);
    
    event!(Level::INFO, "Variable init on runtime complete");

    info!("[INFO] server started on port : {:?}", &port);
    //let buffer_service  = Arc::new(Vec::new());
    
    let (tx_file_data, mut rx_file_data) = mpsc::unbounded_channel::<obj::FileData>();
    let (child_tx, mut child_rx) = watch::channel::<String>("init".to_string());

    //------------------------------------------------------------------------------------------------
    //attempt_read_file(rx);

    task::spawn_blocking(move || {

        while let Some(fileObj) = rx_file_data.blocking_recv() {

            tracing::info!(
                "Attempting to acquire inside spawn blocking: {:?}",
                //Utc::now().format("%D %H:%M:%S"),
                &fileObj.filePath
            );            

            let file_name = format!(
                "./output/{}.{}",
                fileObj.uuid.clone(),
                "webm"
            );

            child_tx.send(fileObj.uuid.clone()).unwrap();

            let ffmpeg_process = 

            // detect the platform from and work accordingly
                if cfg!(target_os = "linux") 
            {
                match Command::new("ffmpeg")
                    .args(["-i", &fileObj.filePath.clone(), &file_name])
                    .output()
                    {
                        Ok(out) => {
                            
                            if out.status.success() {

                                String::from_utf8(out.stdout)

                            }
                            else {
                                String::from_utf8(out.stderr)
                            }

                        }
                        Err(e ) => {
                                tracing::log::error!(" Process cannot be started EXITED: {:?}", e);
                                continue;
                        }
                    }

            } else {
                //switch to the linux context for the
               panic!("THE PLATFORM IS NOT SUPPORTED");
            };
            //send the status of child process
            //let status = ffmpeg.
            tracing::log::info!(" File created :{}", &file_name);

            //let out = ffmpeg_process.expect("ERROR!").stdout;

            println!(
                "[{}] ouput of child process :{:?}",
                Utc::now().format("%D %H:%M:%S"),
                ffmpeg_process.unwrap()
            );

            child_tx.send("NULL".to_string()).unwrap();
        }
    });


    let core_route = warp::path("upload");
    //------------------------------------------------------------------------------------------------
    let upload = 
        core_route
        .and(warp::get())
        .and(handler::user_request_body())
        .and(warp::multipart::form().max_length(1024*32*32254))
        .and(with_tx(tx_file_data.clone()))
        //.and(user_request_body)
        .and_then(handler::upload);
   

    let check_queue = 
        core_route
        .and(warp::path("check"))
        .and(warp::get())
        .map(move || {
            let stat = child_rx.borrow().clone();
            format!("This process is in exec: {:?}", stat)
        });
    
    
    let fetch_file = core_route
        .and(warp::path("download"))
        .and(warp::get())
        //.and(handler::user_request_body())
        //.and_then(|_| async {} )
        .and(warp::path::param( ))
        .map(  handler::file_response);       

        /*
    let file_serve = 

    let server-event = warp::path("push").and(warp::sse()).map(|sse: warp::sse::Sse)  {
        let events = futures::stream::iter_ok::<_, std::io::Error>(vec![]);
    })

*/
    let def_route = warp::service(
        warp::any().or(upload).or(check_queue).or(fetch_file) );
    //.recover(handler::handle_rejection)

    //------------------------------------------------------------------------------------------------

    let def_route_gen = def_route.clone();

    let make_svc = make_service_fn(move |_| {
        let def_route = def_route_gen.clone();

        async move {
            info!("Hyper interior service started");
            Ok::<_, Infallible>(def_route) //return the value generated
        }
    });

    let server = Server::bind(&([0, 0, 0, 0], port.clone()).into())
        //.http2_only(true)
        .serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", &e);
        error!("server falid to continue: {:?}", &e);
    }
}
