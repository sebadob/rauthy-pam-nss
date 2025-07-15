use crate::PROXY_SOCKET;
use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper::http::{Method, Request, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;

// not a very efficient impl, because it opens the TCP stream each time, but
// absolutely fine when we only need to do a single request all the time anyway
pub async fn get(path: &str) -> anyhow::Result<(StatusCode, Bytes)> {
    let stream = TokioIo::new(UnixStream::connect(PROXY_SOCKET).await?);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {err:?}");
        }
    });

    let body: http_body_util::Empty<Bytes> = http_body_util::Empty::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(body)?;

    let res = sender.send_request(request).await?;
    let status = res.status();
    let body = res.collect().await?.to_bytes();

    Ok((status, body))
}
