extern crate mini_http;
extern crate module_interface;

use std::ffi::CString;

pub struct LoggerRequest;

impl module_interface::InputModule for LoggerRequest {
    fn compute(&self, request: &mini_http::request::Request) -> CString {
        println!("incoming request: {:?}", request);
        CString::new("logged").unwrap()
    }
}

