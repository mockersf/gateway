extern crate futures;
extern crate futures_cpupool;
extern crate num_cpus;
extern crate tokio_proto;
extern crate tokio_service;
extern crate url;
extern crate bytes;
extern crate tokio_io;
extern crate tokio_core;
extern crate httparse;

use std::sync::mpsc;
use std::thread;
use std::net::ToSocketAddrs;
use std::{io, str};
use futures::future::{self, Either};
use futures::{BoxFuture, Future};
use tokio_proto::TcpServer;
use tokio_service::Service;
use tokio_core::net::TcpStream;

mod request;
mod response;
mod mini_http;
use request::Request;
use response::Response;
use mini_http::Http;

struct Gateway {
    remote_handle: tokio_core::reactor::Remote,
}

impl Service for Gateway {
    type Request = Request;
    type Response = Response;
    type Error = std::io::Error;
    type Future = Either<future::Ok<Response, io::Error>, BoxFuture<Response, io::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match req.headers()
                  .into_iter()
                  .find(|header| header.0 == "Host")
                  .map(|header| header.1) {
            Some("127.0.0.1:8080") => Either::A(future::ok(self.internal(req))),
            Some("localhost:8080") => Either::B(self.forward(req)),
            _ => {
                let mut resp = Response::new();
                resp.status_code(404, "Not Found");
                Either::A(future::ok(resp))
            }
        }
    }
}

impl Gateway {
    fn internal(&self, req: Request) -> Response {
        println!("internal request, do some config: {:?}", req);
        let mut resp = Response::new();
        resp.header("Content-Type", "text/plain")
            .body("configure me");
        resp
    }

    fn forward(&self, req: Request) -> BoxFuture<Response, io::Error> {
        //        println!("forward request: {:?}", req);
        //        println!("data: {:?}", req.data);
        let (tx, rx) = futures::oneshot();

        self.remote_handle
            .spawn(move |handle| {
                let addr = "127.0.0.1:1026".to_socket_addrs().unwrap().next().unwrap();
                TcpStream::connect(&addr, handle)
                    .and_then(move |socket| tokio_io::io::write_all(socket, req.data))
                    .and_then(|(socket, _request)| tokio_io::io::read_to_end(socket, Vec::new()))
                    .and_then(|(_socket, data)| {
                                  let a = tx.send(data);
                                  a.unwrap();
                                  Ok(())
                              })
                    .map_err(|_| ())
            });

        let data = match rx.wait() {
            Ok(data) => data,
            Err(_) => {
                let v: Vec<u8> = vec![];
                v
            }
        };
        future::ok(Response::raw(data)).boxed()
    }
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut core = match tokio_core::reactor::Core::new() {
            Ok(core) => core,
            Err(err) => {
                tx.send(Err(err)).expect("Channel was closed early");
                return;
            }
        };

        tx.send(Ok(core.remote()))
            .expect("Channel was closed early");

        loop {
            core.turn(None);
        }
    });

    let remote = rx.recv().unwrap();
    let remote: tokio_core::reactor::Remote = remote.unwrap();
    let mut srv = TcpServer::new(Http, addr);
    srv.threads(num_cpus::get());

    srv.serve(move || Ok(Gateway { remote_handle: remote.clone() }))
}
