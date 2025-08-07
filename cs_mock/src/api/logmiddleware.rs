use poem::{async_trait, Endpoint, IntoResponse, Middleware, Request, Result};
use tracing::{event, Level};



#[derive(Clone)]
pub struct LogRequest;

impl <T: Endpoint> Middleware<T> for LogRequest {
    type Output = LogRequestImpl<T>;

    fn transform(&self, ep: T) -> Self::Output {
        LogRequestImpl { ep }
    }
}

pub struct LogRequestImpl<E> {
    ep: E,
}

#[async_trait]
impl <E: Endpoint> Endpoint for LogRequestImpl<E> {
    type Output = E::Output;
    async fn call(&self, req: Request) -> Result<Self::Output> {
        // Log request details before calling the inner endpoint
        event!( Level::DEBUG,"Request: {:?}", req);

        // Call the inner endpoint
        let res = self.ep.call(req).await;
        res
    }
    // #[doc = " Get the response to the request."]
    // #[must_use]
    // #[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
    // fn call<'life0,'async_trait>(&'life0 self,req:Request) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<Self::Output> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
    //     self.ep.call(req).await
    // }
}