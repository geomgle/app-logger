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
use color_eyre::eyre::{Result, WrapErr};
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

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });
    let server = Server::bind(&addr).serve(make_svc);
    println!("app log is running at: 3000");
    Ok(server.await.wrap_err("failed to running server")?)
}

pub fn connect_docker() -> Result<Docker> {
    Docker::connect_with_socket_defaults()
        .wrap_err("failed to connect docker socket")
}

fn get_log_stream(
    docker: &Docker,
) -> impl Stream<Item = Result<bytes::Bytes, bollard::errors::Error>> {
    let options = Some(LogsOptions::<String> {
        stdout: true,
        ..Default::default()
    });

    docker.logs("app", options).map(|log| log.map(|output| output.into_bytes()))
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        // Stream a file from a disk
        (&Method::GET, "/logs") => {
            let docker = connect_docker()?;
            let stream = get_log_stream(&docker);
            let s = Body::wrap_stream(stream);
            let response = Response::new(s);
            return Ok(response);
        }

        // 404 not found
        _ => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::NOT_FOUND;
            return Ok(response);
        }
    };
}
