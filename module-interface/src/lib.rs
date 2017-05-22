extern crate mini_http;

use std::ffi::CString;

pub trait InputModule {
    fn compute(&self, &mini_http::request::Request) -> CString; //where Self: Sized;
}
