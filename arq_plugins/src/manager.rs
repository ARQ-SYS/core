use std::ffi::OsStr;

use libloading::{Library, Symbol};
use rocket::Route;


use tracing::{info, debug};
use crate::{component::Component, middleware::MiddlewareComponent};

use super::middleware::DynFairing;


/// This struct is used to orchestrate the loading of the components and middlewares.
/// It will be used by the CORE to load the components and middlewares.
pub struct PluginManager {
    components: Vec<Box<dyn Component>>,
    middlewares: Vec<Box<dyn MiddlewareComponent>>,
    loaded_libs: Vec<Library>
}

impl PluginManager {
    /// Constructs a new ComponentManager.
    pub fn new() -> Self {
        PluginManager {
            components: Vec::new(),
            middlewares: Vec::new(),
            loaded_libs: Vec::new()
        }
    }
    /// Loads the component from the given path.
    pub unsafe fn load_components<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<(), libloading::Error> {

        type ComponentConstructor = fn() -> *mut dyn Component;

        debug!("Loading component from {}", filename.as_ref().to_string_lossy());
        let lib = Library::new(filename.as_ref())?;

        self.loaded_libs.push(lib);               
        let lib = self.loaded_libs.last().unwrap(); // This is safe because we just pushed it.

        let component_constructor: Symbol<ComponentConstructor> = lib.get(b"_arq_component_constructor")?;
        let raw = component_constructor();
        let component = Box::from_raw(raw);
        info!("Loaded component: {}", component.name());
        component.on_component_load();
        self.components.push(component);

        Ok(())
    }

    /// This functions unloads the components and middlewares from ComponentManager.
    /// This wont unload the components and middlewares from the CORE, when it's already running.
    pub fn unload(&mut self) {
        info!("Unloading middleware");
        for middleware in self.middlewares.drain(..) {
            debug!("Unloading middleware: {}", middleware.name());
            middleware.on_middleware_unload();
        }
        
        info!("Unloading components");
        for component in self.components.drain(..) {
            debug!("Unloading middleware: {}", component.name());
            component.on_component_unload();
        }

        for lib in self.loaded_libs.drain(..) {
            drop(lib)
        }
    }
    /// This function returns the routes that should be mounted by CORE.
    pub fn get_routes(&self) -> Vec<Route> {

        let mut out = Vec::new();
        for comp in &self.components {
            let raw = comp.routes();
            unsafe {
                let complete = Vec::from_raw_parts(raw.0, raw.1, raw.2);
                out.extend(complete);
            }
        }
        out
    }
    /// Return all loaded middleware to be attached
    pub fn get_middlewares(&self) -> Vec<DynFairing> {
        let mut out = Vec::new();
        for middleware in &self.middlewares {
            let raw = middleware.middlewares();
            unsafe {
                let complete = Vec::from_raw_parts(raw.0, raw.1, raw.2);
                let mut loadable: Vec<DynFairing> = Vec::new();
                for mid in complete {
                    loadable.push(DynFairing::from(mid))
                }
                out.extend(loadable);
            }
        }
        out
    }

    // Loads the middleware from the given path.
    pub unsafe fn load_middleware<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<(), libloading::Error> {

        type MiddlewareConstructor = fn() -> *mut dyn MiddlewareComponent;

        debug!("Loading middleware from {}", filename.as_ref().to_string_lossy());
        let lib = Library::new(filename.as_ref())?;

        self.loaded_libs.push(lib);               
        let lib = self.loaded_libs.last().unwrap(); // This is safe because we just pushed it.

        let middleware_constructor: Symbol<MiddlewareConstructor> = lib.get(b"_arq_middleware_constructor")?;
        let raw = middleware_constructor();
        let middleware = Box::from_raw(raw);
        info!("Loaded middleware: {}", middleware.name());
        middleware.on_middleware_load();
        self.middlewares.push(middleware);

        Ok(())
    }

}

