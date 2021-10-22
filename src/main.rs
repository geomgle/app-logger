#![warn(rust_2018_idioms)]
#![allow(unused, dead_code, unused_imports, unused_doc_comments)]
#![allow(clippy::new_without_default)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::should_implement_trait)]
use std::{convert::Infallible, net::SocketAddr, process::Stdio};

use bollard::{
    container::{LogOutput, LogsOptions},
    Docker,
};
use bytes::BytesMut;
use futures::stream::{Stream, StreamExt};
use futures_util::{TryFutureExt, TryStreamExt};
use hyper::{
    service::{make_service_fn, service_fn},
    Body,
    Method,
    Request,
    Response,
    Server,
    StatusCode,
};
use tokio::{fs::File, process::Command};
use tokio_util::codec::{BytesCodec, FramedRead};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let docker = connect_docker();
    let mut stream = get_log_stream(&docker);
    while let Some(log) = stream.next().await {
        println!("{:#?}", log.unwrap().into_bytes());
    }
    Ok(())
}

pub fn connect_docker() -> Result<Docker> {
    Ok(Docker::connect_with_socket_defaults().map_err(|e| {
        eprintln!("failed to connect docker socket: {:#?}", e);
        e
    })?)
}

fn get_log_stream(
    docker: &Docker,
) -> impl Stream<Item = std::result::Result<LogOutput, bollard::errors::Error>>
{
    let options = Some(LogsOptions::<String> {
        stdout: true,
        ..Default::default()
    });

    docker.logs("app", options)
}
