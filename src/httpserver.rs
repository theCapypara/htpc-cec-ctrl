use cec_rs::{CecConnection, CecLogicalAddress};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use system_shutdown::shutdown;
use tokio::net::TcpListener;

const METHOD: Method = Method::POST;
const TV: CecLogicalAddress = CecLogicalAddress::Tv;

async fn service(
    req: Request<hyper::body::Incoming>,
    cec: Arc<CecConnection>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&METHOD, "/tv-on") => match cec.send_power_on_devices(TV) {
            Ok(_) => info!("powered on TV via HTTP"),
            Err(err) => error!("HTTP power on TV error: {err:?}"),
        },
        (&METHOD, "/tv-off") => match cec.send_standby_devices(TV) {
            Ok(_) => info!("send TV to standby via HTTP"),
            Err(err) => error!("HTTP standby TV error: {err:?}"),
        },
        (&METHOD, "/pc-off") => match shutdown() {
            Ok(_) => info!("shutdown via HTTP"),
            Err(err) => error!("shutdown via HTTP error: {err:?}"),
        },
        _ => {}
    }
    Ok(Response::new(empty()))
}

pub async fn run_server(cec_connection: CecConnection) {
    let cec_connection = Arc::new(cec_connection);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let cec_connection = cec_connection.clone();
        if let Ok((stream, _)) = listener.accept().await {
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| service(req, cec_connection.clone())),
                    )
                    .await
                {
                    error!("Error serving connection: {:?}", err);
                }
            });
        } else {
            error!("Error accepting connection");
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
