use std::sync::Arc;

use lib;

use module_interface::{InputModule, ModuleResponse};
use hyper::server::Request;

#[derive(Clone)]
pub struct LoadedInputModule {
    lib: Arc<lib::Library>,
    compute: lib::os::unix::Symbol<extern "Rust" fn(&Request) -> ModuleResponse>,
}

impl LoadedInputModule {
    pub fn load(path: &str) -> LoadedInputModule {
        let lib = lib::Library::new(path).unwrap();
        let compute = unsafe {
            let func: lib::Symbol<extern "Rust" fn(&Request) -> ModuleResponse> =
                lib.get(b"compute\0").unwrap();
            func.into_raw()
        };
        println!("Loaded {:?}", path);
        LoadedInputModule {
            lib: Arc::new(lib),
            compute,
        }
    }
}

impl InputModule for LoadedInputModule {
    fn compute(&self, req: &Request) -> ModuleResponse {
        let compute = &self.compute;
        compute(req)
    }
}
