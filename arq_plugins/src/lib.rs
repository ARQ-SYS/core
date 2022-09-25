//! # Overview
//! Arq Plugins is a crate that will allow you to create plugins for Arq Core  
//! This will be a simple tutorial as well as a technical refference, if you'd like to go in yourself
//! It uses Rust FFI under the hood with the C ABI, so seqfaults can happen. I've *hopefully* made it foolproof enough,  
//! that you hopefully won't encounter any. But in a case that it happens, don't hesisate to create an issue [here](https://github.com/ARQ-SYS/ARQ)



pub mod component;
pub mod manager;
pub mod middleware;

pub mod prelude {
    pub use crate::component::*;
    pub use crate::manager::*;
    pub use crate::middleware::*;
}

/// This macro is used to declare a component.
/// It must be used excatly once per project.
/// This must be used alongside the `ComponentFactory::export` method.
/// This means that you can have only one component per project, but as many paths as you want.
#[macro_export]
macro_rules! declare_component {
    ($plugin_type: ty, $constructor: path) => {
        #[no_mangle]
        pub extern "C" fn _arq_component_constructor() -> *mut dyn Component {
            use arq_components::pluggable::component::Component;

            let constructor: fn() -> $plugin_type = $constructor;
            let objet = constructor();
            let boxed: Box<dyn Component> = Box::new(objet);
            Box::into_raw(boxed)
        }
    }
}


/// This macro is used to declare a middleware.
/// It must be used excatly once per project.
/// This must be used alongside the `MiddlewareFactory::export` method.
/// This means that you can have only one component per project, but as many paths as you want.
#[macro_export]
macro_rules! declare_middleware {
    ($plugin_type: ty, $constructor: path) => {
        #[no_mangle]
        pub extern "C" fn _arq_middleware_constructor() -> *mut dyn MiddlewareComponent {
            use arq_components::pluggable::middleware::MiddlewareComponent;

            let constructor: fn() -> $plugin_type = $constructor;
            let objet = constructor();
            let boxed: Box<dyn MiddlewareComponent> = Box::new(objet);
            Box::into_raw(boxed)
        }
    }
}