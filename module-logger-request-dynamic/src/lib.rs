extern crate mini_http;

use std::ffi::CString;

#[no_mangle]
pub fn compute(request: &mini_http::request::Request) -> CString {
    println!("incoming data: {:?}", request.data);
    CString::new("logged").unwrap()
}
