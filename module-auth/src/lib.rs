extern crate hyper;
extern crate module_interface;

use hyper::StatusCode;
use hyper::server::Response;

use module_interface::ModuleResponse;

#[no_mangle]
pub extern "Rust" fn compute(request: &hyper::server::Request) -> ModuleResponse {
    match request.headers().get::<hyper::header::Authorization<hyper::header::Basic>>().cloned() {
        Some(_) => ModuleResponse::Noop,
        None => {
            println!("missing authentication, not authorized");
            ModuleResponse::Stop(Response::new().with_status(StatusCode::Unauthorized))
        }
    }
}
