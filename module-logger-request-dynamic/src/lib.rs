extern crate hyper;
extern crate module_interface;

use module_interface::ModuleResponse;

#[no_mangle]
pub fn compute(request: &hyper::server::Request) -> ModuleResponse {
    println!("incoming query: {:?} - {:?}", request.path(), request.headers());
    ModuleResponse::Ignore
}
