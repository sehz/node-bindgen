use std::ptr;

use log::debug;
use inventory::iter;

use crate::Property;
use crate::val::JsExports;
use crate::sys::napi_value;
use crate::sys::napi_env;
use crate::NjError;

type ClassCallback = fn(&mut JsExports) -> Result<(), NjError>;

enum NapiRegister {
    Property(Property),
    Callback(ClassCallback),
}

inventory::collect!(NapiRegister);


#[macro_export]
macro_rules! submit_property {
    ($property:expr) => {
        inventory::submit! {
            NapiRegister::Property(value)
        }
    };
}

#[macro_export]
macro_rules! submit_register_callback {
    ($property:expr) => {
        inventory::submit! {
            NapiRegister::Callback(value)
        }
    };
}



#[no_mangle]
pub extern "C" fn init_modules(env: napi_env, exports: napi_value) -> napi_value {
    debug!("initializing modules");

    let mut js_exports = JsExports::new(env, exports);
    let mut prop_builder = js_exports.prop_builder();

    for register in iter::<NapiRegister> {
        match register {
            NapiRegister::Property(property) => {
                debug!("registering property: {:#?}", property);
                prop_builder.mut_append(property.to_owned());
            }
            NapiRegister::Callback(callback) => {
                debug!("invoking register callback");
                if let Err(err) = callback(&mut js_exports) {
                    panic!("error invoking JS callback: {}", err);
                }
            }
        }
    }

    js_exports
        .define_property(prop_builder)
        .expect("property should not fail");

    exports
}

#[crate::ctor]
fn init_module() {
    use crate::c_str;
    use crate::sys::NAPI_VERSION;
    use crate::sys::napi_module;
    use crate::sys::napi_module_register;

    static mut _MODULE: napi_module = napi_module {
        nm_version: NAPI_VERSION as i32,
        nm_flags: 0,
        nm_filename: c_str!("lib.rs").as_ptr() as *const ::std::os::raw::c_char,
        nm_register_func: Some(init_modules),
        nm_modname: c_str!("rust_module").as_ptr() as *const ::std::os::raw::c_char,
        nm_priv: ptr::null_mut(),
        reserved: [
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        ],
    };

    unsafe {
        napi_module_register(&mut _MODULE);
    }
}
