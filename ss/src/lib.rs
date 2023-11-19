// hyper = { version = "1", features = ["full"] }
// tokio = { version = "1", features = ["full"] }
// http-body-util = "0.1"
// hyper-util = { version = "0.1", features = ["full"] }
//
/*

	port
	Option<certificates>
	f: request -> +
		HTTP response
		WebSocket handler
			f: Message -> Option<Message>
		WebSocket handler 2
			f: WS sink, WS stream ->

 */

use hyper::Request;
use hyper::Response;
//use std::convert::Infallible;
use tungstenite::protocol::Message;
use hyper::body::Incoming;
use http_body_util::Full;
//use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper::server::conn::http1::Builder;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
//use std::pin::Pin;
//use std::future::Future;




pub enum Potato {
	HTTPResponse(Response<Full<Bytes>>),
	WebSocketHandler(Box<dyn (Fn(Message) -> Option<Message>) + Send>),
}

pub async fn serve_blocking<F>(
	port: u16,
	f: F
)
where F: (Fn(&mut Request<Incoming>) -> Potato) + std::marker::Send + 'static + Clone + std::marker::Sync
{
	println!("making listener");
	let addr = SocketAddr::from(([0, 0, 0, 0], port));
	let listener = TcpListener::bind(addr).await.unwrap();
	println!("made listener");
	loop {
		println!("listener looking for stream");
		let (stream, _) = listener.accept().await.unwrap();
		println!("listener got stream");
		let f = (&f).clone();
		tokio::task::spawn(async move {
			let f = (&f).clone();
			let io = hyper_util::rt::TokioIo::new(stream);
			if let Err(err) = Builder::new()
			.serve_connection(io, service_fn(
				|mut req: Request<hyper::body::Incoming>| async {
					let f = (&f).clone();
					match f(&mut req) {
						Potato::HTTPResponse(r) => { return Ok::<_, hyper::http::Error>(r); },
						Potato::WebSocketHandler(fws) => {
							let headers = req.headers();
							let reqkey = headers["Sec-WebSocket-Key"].clone();
							let retkey = tungstenite::handshake::derive_accept_key(reqkey.as_bytes());
							tokio::task::spawn(async move {
								match hyper::upgrade::on(&mut req).await {
									Ok(upgraded) => {
										let parts: hyper::upgrade::Parts<TokioIo<TcpStream>> = upgraded.downcast().unwrap();
										let stream = parts.io.into_inner();
										//let mut wsock = tokio_tungstenite::accept_async(stream).await.unwrap();
										let mut wsock = tokio_tungstenite::WebSocketStream::from_raw_socket(
											stream,
											tungstenite::protocol::Role::Server,
											None).await;
										use futures_util::sink::SinkExt;
										use futures_util::stream::StreamExt;
										loop {
											let Some(Ok(val)) = wsock.next().await else {
												println!("oh no.");
												break;
											};
											// val: Message
											let mmsg = { fws(val).clone() };
											if let Some(msg) = mmsg {
												wsock.send(msg).await.unwrap();
											}
										}
									}
									Err(e) => eprintln!("upgrade error: {}", e),
								}
							});
							return Ok(hyper::Response::builder()
							.status(101)
							.header("Connection", "Upgrade")
							.header("Upgrade", "websocket")
							.header("Sec-WebSocket-Accept", retkey)
							.body(Full::new(Bytes::from("")))?);
						}
					}
				}
			))
			.with_upgrades()
			.await
			{
				println!("Error serving connection: {:?}", err);
			}
		});
	}
}


// private stuff now ////////////////////////////////////

