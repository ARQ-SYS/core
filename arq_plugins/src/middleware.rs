
use std::{mem, any::Any};
use rocket::{fairing::Fairing, Request, Data, Response};



/// This trait is used when one wants to add middleware to the ARQ_CORE.
/// This acts as a request guard and so should be used with caution, as it could interfiere with the base CORE routes.
pub trait MiddlewareComponent: Any + Send + Sync {
    /// This function should return the name of the middleware.
    fn name(&self) -> &'static str;
    /// This function is fired before the middleware is mounted by CORE.
    fn on_middleware_load(&self) {}
    /// This function is fired after the middleware is unmounted by CORE.
    fn on_middleware_unload(&self) {}
    /// This function should return the routes that should be mounted by CORE.
    /// The returning should be handled by the `arq_components::pluggable::middleware::MiddlewareFactory`.
    fn middlewares(&self) -> (*mut Box<dyn Fairing>, usize, usize);
}


/// This struct is used to export the component's middlewares to the CORE.
/// It must be used alongside the macro `declare_middleware!`, so the ComponentManager can find the MiddlewareComponent.
pub struct MiddlewareFactory {
    pub middlewares: Vec<Box<dyn Fairing>>
}

impl MiddlewareFactory {
    /// Construct a new MiddlewareFactory
    pub fn new() -> Self {
        MiddlewareFactory { middlewares: Vec::new() }
    }
    /// Add a single middleware to be exported
    pub fn add_middleware(mut self, middleware: Box<dyn Fairing>) -> Self {
        self.middlewares.push(middleware);
        self
    }
    /// Add multiple middlewares to be exported
    pub fn add_middlewares(mut self, middlewares: Vec<Box<dyn Fairing>>) -> Self {
        self.middlewares.extend(middlewares);
        self
    }
    /// Export all the middlewares to be mounted by CORE
    /// This consumes the factory
    pub fn export(mut self) -> (*mut Box<dyn Fairing>, usize, usize) {

        self.middlewares.shrink_to_fit();
        assert!(self.middlewares.len() == self.middlewares.capacity());

        let ptr = self.middlewares.as_mut_ptr();
        let len = self.middlewares.len();

        mem::forget(self.middlewares);

        (ptr, len, len)

    }

}

/// A struct that will act as a wrapper for `dyn Fairing` since rocket cannot attach `dyn Trait` as a middleware
/// This should only be used by the ComponentManager, users do not need to worry about this
pub struct DynFairing {
    inner: Box<dyn Fairing>
}

impl DynFairing {
    /// Constructs a new DynFairing
    pub fn from(inner: Box<dyn Fairing>) -> Self {
        Self {
            inner
        }
    }   
}
#[allow(unused)]
#[rocket::async_trait]
impl Fairing for DynFairing {
    fn info(&self) -> rocket::fairing::Info {
        self.inner.info()
    }
    async fn on_request(&self, request: &mut Request<'_>, data: &mut Data<'_>) {
        self.inner.on_request(request, data);
    }
    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        self.inner.on_response(request, response);
    }
}