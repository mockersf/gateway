use std::ffi::CString;
use std::sync::Arc;

use lib;

use module_interface::InputModule;
use mini_http::request::Request;

#[derive(Clone)]
pub struct LoadedInputModule {
    lib: Arc<lib::Library>,
    compute: lib::os::unix::Symbol<unsafe extern "C" fn(&Request) -> CString>,
}

impl LoadedInputModule {
    pub fn load(path: &str) -> LoadedInputModule {
        println!("loading {}", path);
        let lib = lib::Library::new(path).unwrap();
        println!("lib: {:?}", lib);
        let compute = unsafe {
            let func: lib::Symbol<unsafe extern "C" fn(&Request) -> CString> =
                lib.get(b"compute\0").unwrap();
            func.into_raw()
        };
        println!("done {:?}", compute);
        LoadedInputModule {
            lib: Arc::new(lib),
            compute,
        }
    }
}

impl InputModule for LoadedInputModule {
    fn compute(&self, req: &Request) -> CString {
        unsafe {
            let compute = &self.compute;
            let res = compute(req);
            println!("done");
            res
        }
    }
}
