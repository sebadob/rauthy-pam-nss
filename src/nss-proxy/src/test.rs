// use crate::PROXY_SOCKET;
// use axum::{
//     body::Body,
//     http::{Method, Request, StatusCode},
// };
// use http_body_util::BodyExt;
// use hyper_util::rt::TokioIo;
// use log::{error, info};
// use std::path::PathBuf;
// use std::time::Duration;
// use tokio::net::UnixStream;
// use tokio::{task, time};
//
// pub fn spawn_tests() {
//     task::spawn(async move {
//         time::sleep(Duration::from_secs(1)).await;
//
//         if let Err(err) = run_tests().await {
//             error!("Error during tests: {err}");
//         }
//     });
// }
//
// pub async fn run_tests() -> anyhow::Result<()> {
//     let path = PathBuf::from(PROXY_SOCKET);
//     let stream = TokioIo::new(UnixStream::connect(path).await?);
//     let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await?;
//     tokio::task::spawn(async move {
//         if let Err(err) = conn.await {
//             println!("Connection failed: {err:?}");
//         }
//     });
//
//     // root path
//     let request = Request::builder()
//         .method(Method::GET)
//         .uri("/")
//         .body(Body::empty())?;
//
//     let response = sender.send_request(request).await?;
//
//     assert_eq!(response.status(), StatusCode::OK);
//
//     let body = response.collect().await?.to_bytes();
//     let body = String::from_utf8(body.to_vec())?;
//     assert_eq!(body, "Hello, World!");
//
//     // getent path
//     let request = Request::builder()
//         .method(Method::GET)
//         .uri("/getent")
//         .body(Body::empty())?;
//
//     let response = sender.send_request(request).await?;
//
//     assert_eq!(response.status(), StatusCode::OK);
//
//     let body = response.collect().await?.to_bytes();
//     let body = String::from_utf8(body.to_vec())?;
//     assert_eq!(body, "GETENT");
//
//     info!("All tests successful");
//
//     Ok(())
// }
