extern crate hyper;
extern crate module_interface;

use module_interface::ModuleResponse;

#[no_mangle]
pub fn compute(request: &hyper::server::Request) -> ModuleResponse {
    match request.headers().get::<hyper::header::Authorization<hyper::header::Basic>>().cloned() {
        Some(_) => ModuleResponse::Ignore,
        None => {
            println!("missing authentication");
            ModuleResponse::Stop
        }
    }
}
