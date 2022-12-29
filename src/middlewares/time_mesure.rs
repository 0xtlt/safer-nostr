use actix_web::{
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct TimeMesure;

impl<S: 'static> Transform<S, ServiceRequest> for TimeMesure
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = TimeMesureMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TimeMesureMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct TimeMesureMiddleware<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for TimeMesureMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = std::time::Instant::now();
        let service = self.service.clone();
        let url = req.uri().path().to_owned();
        let fut = service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let end = std::time::Instant::now();
            let duration = end.duration_since(start);
            let duration = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;

            println!("{url}: {duration}s");

            Ok(res)
        })
    }
}
