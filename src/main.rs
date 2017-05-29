extern crate futures;

extern crate tokio_core;
extern crate tokio_proto;

extern crate hyper;

#[macro_use]
extern crate lazy_static;

extern crate libloading as lib;

extern crate module_interface;

use std::cell::Cell;
use std::sync::{Arc, Mutex};

use futures::Future;

use tokio_core::reactor::Remote;

use hyper::StatusCode;
use hyper::header::Host;
use hyper::server::{Http, Service, Request, Response};
use hyper::Client;

use module_interface::InputModule;

mod lib_loader;
use lib_loader::LoadedInputModule;

lazy_static! {
    static ref REMOTE: Arc<Mutex<Cell<Option<Remote>>>> = Arc::new(Mutex::new(Cell::new(None)));
}

thread_local! {
    static CLIENT: Client<hyper::client::HttpConnector> =
            Client::new(&REMOTE.lock().unwrap().get_mut().clone().unwrap().handle().unwrap());
}

lazy_static! {
    static ref INPUT_MODULES: Arc<Mutex<Vec<LoadedInputModule>>> = Arc::new(Mutex::new(vec![]));
}

#[derive(Clone, Copy)]
struct Config {
    target: &'static str,
    target_port: u16,
}

struct Gateway {
    config: Config,
}

impl Gateway {
    fn forwarded_request(&self, req: Request) -> Request {
        let url = match req.uri().query() {
                Some(qp) => {
                    format!("http://{}:{}{}?{}",
                            self.config.target,
                            self.config.target_port,
                            req.path(),
                            qp)
                }
                None => {
                    format!("http://{}:{}{}",
                            self.config.target,
                            self.config.target_port,
                            req.path())
                }
            }
            .parse::<hyper::Uri>()
            .unwrap();
        let mut forwarded_request = req;
        forwarded_request
            .headers_mut()
            .set::<Host>(Host::new(self.config.target, self.config.target_port));
        forwarded_request.set_uri(url);
        forwarded_request
    }
}

impl Service for Gateway {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = hyper::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match req.headers().get::<hyper::header::Host>().cloned() {
            Some(ref host) if host.hostname() == "127.0.0.1" => self.internal(req),
            Some(ref host) if host.hostname() == "localhost" => self.forward(req),
            _ => futures::future::ok(Response::new().with_status(StatusCode::NotFound)).boxed(),
        }
    }
}

impl Gateway {
    fn internal(&self, req: Request) -> Box<Future<Item = Response, Error = hyper::Error>> {
        println!("internal request, do some config: {:?}", req);
        futures::future::ok(Response::new().with_status(StatusCode::Accepted)).boxed()
    }

    fn forward(&self, req: Request) -> Box<Future<Item = Response, Error = hyper::Error>> {
        for module in INPUT_MODULES.lock().unwrap().iter() {
            match module.compute(&req) {
                module_interface::ModuleResponse::Noop => (),
                module_interface::ModuleResponse::Stop(resp) => return futures::future::ok(resp).boxed()
            }
        }

        let forwarded_request = self.forwarded_request(req);
        CLIENT.with(|client| {
            Box::new(client
                         .request(forwarded_request)
                         .map(|res| {
                                  let res: Response = res;
                                  if res.status() != StatusCode::Ok {
                                      println!("{:?}", res);
                                  }
                                  res
                              })
                         .or_else(|err| {
                println!("{:?}", err);
                futures::future::ok(Response::new().with_status(StatusCode::ServiceUnavailable))
            }))
        })

    }
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();

    let config = Config {
        target: "localhost",
        target_port: 8080,
    };

    {
        let mut modules_lock = INPUT_MODULES.lock().unwrap();
        let module_path = "module-logger-request-dynamic/target/release/libmodule_logger_request.dylib";
        let logger_module = lib_loader::LoadedInputModule::load(module_path);
        modules_lock.push(logger_module);
        let module_path = "module-auth/target/release/libmodule_auth.dylib";
        let auth_module = lib_loader::LoadedInputModule::load(module_path);
        modules_lock.push(auth_module);
    }

    let server = Http::new()
        .bind(&addr, move || {
            Ok(Gateway { config })
        })
        .unwrap();
    println!("Listening on http://{} with 1 thread.",
             server.local_addr().unwrap());

    REMOTE
        .lock()
        .unwrap()
        .set(Some(server.handle().remote().clone()));
    server.run().unwrap();
}
