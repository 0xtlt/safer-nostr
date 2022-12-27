use crate::systems::url::parse_query_string;
use actix_web::{
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct Validate;

impl<S: 'static> Transform<S, ServiceRequest> for Validate
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = ValidateMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ValidateMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct ValidateMiddleware<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for ValidateMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let search_params = parse_query_string(req.request().query_string());
        let svc = self.service.clone();

        Box::pin(async move {
            if crate::systems::security::check_access(
                search_params.get("pubkey"),
                search_params.get("sig"),
                search_params.get("time"),
                search_params.get("uniq"),
            )
            .await
            {
                let res = svc.call(req).await?;
                Ok(res)
            } else {
                let (request, _pl) = req.into_parts();

                let response = HttpResponse::Found().body("Access denied");
                let res: ServiceResponse = ServiceResponse::new(request, response);
                Ok(res)
            }
        })
    }
}
