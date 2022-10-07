//! # Overview
//! Arq Plugins is a crate that will allow you to create plugins for Arq Core.  
//! This will be a simple tutorial as well as a technical refference, if you'd like to go in yourself.  
//! It uses Rust FFI under the hood with the C ABI, so seqfaults can happen.  I've *hopefully* made it foolproof enough,  
//! that you hopefully won't encounter any. But in a case that it happens, don't hesisate to create an issue [here](https://github.com/ARQ-SYS/ARQ)
//! # Creating Plugins
//! ## Prerequisites
//! To create a plugin for ARQ, you first need to create a library crate with `cargo new <plugin_name> --lib`.  
//! Next, you need to change the output of your crate to cdylib  
//! ```toml
//! / ... /
//! [lib]
//! crate-type = ["cdlib"]
//! ```
//! Then you need to add `arq_plugins` to your dependencies  
//! ```toml
//! / ... /
//! [dependencies]
//! arq_plugins = 0.1.0
//! ```  
//! Now you're all set!
//! ## Writing a simple plugin
//! First we need to create a struct that will represent our plugin. 
//! There are two types of possible things we could export - Component (just rocket paths) and Middleware (just rocket fairings).  
//! For this example we'll create a Component.
//! ```rust
//! // Don't forget to derive Default!
//! #[derive(Default)]
//! pub struct MyComponent;
//! ```
//! 
//! <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
//!     <strong>Warning:</strong> And and All state should be handled by rocket Guards.  The Component struct shouldn't have any fields.   
//! </p>
//! 
//! A struct like this won't help us much. To be exportable, we need to implement a trait - `Component`.  
//! ```rust
//! pub trait Component {
//!    fn name(&self) -> &'static str;
//!    fn on_component_load(&self) {}
//!    fn on_component_unload(&self) {}
//!    fn routes(&self) -> (*mut Route, usize, usize);
//! }
//! ```
//! The `Component` trait exposes four basic functions to be implemented:  
//! - name (required) = specify the name of the component
//! - on_component_load = this event fires when the component is loaded by CORE
//! - on_component_unload = this event fires when the component is unloaded by CORE
//! - routes (required) = this is the function that will expose all your routes to CORE  
//! Don't be scared of the weird routes function signtature, the exporting is will be handled nicley
//! ### Implementing Component
//! To implement component trait on your struct you really only need a name, and all your routes.
//! To export routes, we've created a helper struct - `ComponentFactory`.  
//! This currently exists just to export the paths, and will be removed in the future - replaced by a macro.  
//! ```rust
//! #[get("/hello")]
//! fn greet() -> &'static str {
//!     "Hello friend!"
//! }
//! 
//! impl Component for MyComponent {
//! 
//!     fn name(&self) -> &'static str { "MyComponent" }
//!     fn routes(&self) -> (*mut Route, usize, usize) {
//!         let mut factory = ComponentFactory::new();
//!         factory.add_route(greet);
//!         // Now we export
//!         factory.export()
//!         // And it's done
//!     } 
//! 
//! }
//! ```
//! This makes all the routes visible to CORE.  
//! But we're not done yet, as core can't see our Component. What we have to do, is add a `declare_component!()` macro call like so
//! ```rust
//! / ... / 
//! declare_component!(MyComponent, MyComponent::default)
//! ```
//! After this, your plugin should be loadable by CORE
//! If you need more resources, take a look at example plugins


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