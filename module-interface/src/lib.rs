extern crate hyper;

use hyper::server::Request;
use hyper::server::Response;

pub trait InputModule {
    fn compute(&self, &Request) -> ModuleResponse;
}


#[derive(Debug)]
pub enum ModuleResponse {
    Stop(Response),
    Noop,
}
