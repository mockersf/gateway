extern crate hyper;

use hyper::server::Request;

use std::ffi::CString;

pub trait InputModule {
    fn compute(&self, &Request) -> ModuleResponse;
}


#[derive(Debug)]
pub enum ModuleResponse {
    Stop,
    Ignore,
}
