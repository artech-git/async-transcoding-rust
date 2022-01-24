mod handler;
mod obj;

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
//extern crate console_subscriber;

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

    let port: u16 = env!("SRV_PORT").parse::<u16>().unwrap();
    println!("server started on port : {:?}", &port);
    event!(Level::INFO, "something has happened!");

    info!("[INFO] server started on port : {:?}", &port);
    //let buffer_service  = Arc::new(Vec::new());
    
    let (tx, mut rx) = mpsc::unbounded_channel::<obj::FileData>();
    let (child_tx, mut child_rx) = watch::channel::<String>("init".to_string());

    //------------------------------------------------------------------------------------------------
    //attempt_read_file(rx);

    task::spawn_blocking(move || {

        while let Some(fileObj) = rx.blocking_recv() {

            println!(
                "[{}] Attempting to read file: {:?}",
                Utc::now().format("%D %H:%M:%S"),
                &fileObj
            );

            

            let file_name = format!(
                "./output/{}.{}",
                uuid::Uuid::new_v4().to_hyphenated().to_string(),
                "webm"
            );


            child_tx.send(file_name.clone()).unwrap();

		let start_time = Utc::now().time();
            let ffmpeg_process = if cfg!(target_os = "linux") {
                // Windows
                Command::new("ffmpeg")
                    //.arg("ffmpeg")
                    .args(["-i", &fileObj.filePath.clone(), &file_name])
                    .output()
            } else {
                //switch to the linux context for the
                Command::new("ffmpeg").args(["-h"]).output()
            };
		let end_time = Utc::now().time();
            //send the status of child process
            //let status = ffmpeg.
            tracing::log::info!(" File created :{}", &file_name);

            let out = ffmpeg_process.expect("ERROR!").stdout;

            println!(
                "Time: [{}]  ouput of child process :{:?}",
                //Utc::now().format("%D %H:%M:%S"),
                (end_time - start_time).num_minutes(),
		String::from_utf8(out)
            );
            child_tx.send("NULL".to_string()).unwrap();
        }
    });

    let core_route = warp::path("upload");
    //------------------------------------------------------------------------------------------------
    let upload = core_route
        .and(warp::get())
        .and(warp::multipart::form().max_length(50000000000))
        //.and(rate_limit)
        .and(with_tx(tx.clone()))
        .and_then(handler::upload);
    //.with(warp::log("completed the upload"));

    let check_queue = core_route
        .and(warp::path("check"))
        .and(warp::get())
        //.and(with_rx(child_rx.clone()))
        .map(move || {
            let stat = child_rx.borrow().clone();
            format!("This process is in exec: {:?}", stat)
        });
/*
    let file_serve = 

    let server-event = warp::path("push").and(warp::sse()).map(|sse: warp::sse::Sse)  {
        let events = futures::stream::iter_ok::<_, std::io::Error>(vec![]);
    })

*/
    let def_route = warp::service(
        upload.or(check_queue), //.recover(handler::handle_rejection)
    );

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
