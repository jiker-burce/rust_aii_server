use crate::api::token::CurrentUser;
use poem::http::header::AUTHORIZATION;
use poem::{Endpoint, Middleware, Request, Result};

pub struct JwtMiddleware;

impl<E: Endpoint> Middleware<E> for JwtMiddleware {
    type Output = JwtMiddlewareImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        JwtMiddlewareImpl { ep }
    }
}

/// The new endpoint type generated by the TokenMiddleware.
pub struct JwtMiddlewareImpl<E> {
    ep: E,
}

#[poem::async_trait]
impl<E: Endpoint> Endpoint for JwtMiddlewareImpl<E> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        // Just example. You have to make sure your middleware is correct
        if let Some(token) = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.starts_with("Bearer "))
            .map(|value| &value[7..])
        {
            // Decode JWT token
            let result = rc_token::parse_token::<CurrentUser>("123456", token, true)
                .expect("中间件-解析token异常");
            // Attache
            req.extensions_mut().insert(result.1);
        }

        // call the next endpoint.
        self.ep.call(req).await
    }
}