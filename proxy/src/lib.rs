cargo_component_bindings::generate!();

use anyhow::{anyhow, Context, Result};
use bindings::{
    component::middleware,
    exports::wasi::http::incoming_handler::Guest,
    wasi::http::{
        outgoing_handler::{self, OutgoingRequest},
        types::{
            FutureIncomingResponse, Headers, IncomingRequest, IncomingResponse, OutgoingBody,
            OutgoingResponse, ResponseOutparam, Scheme,
        },
    },
};

struct Component;

fn send<F: FnOnce(OutgoingRequest) -> Result<FutureIncomingResponse>>(
    callback: F,
) -> Result<IncomingResponse> {
    let req = OutgoingRequest::new(Headers::new());
    req.set_scheme(Some(&Scheme::Https))
        .map_err(|_| anyhow!("failed to set request scheme"))?;
    req.set_authority(Some("echo.free.beeceptor.com"))
        .map_err(|_| anyhow!("failed to set request authority"))?;
    req.set_path_with_query(Some("/"))
        .map_err(|_| anyhow!("failed to set request path"))?;
    let outgoing_body = req
        .body()
        .map_err(|_| anyhow!("failed to get request body"))?;

    let fut = callback(req)?;
    OutgoingBody::finish(outgoing_body, None)?;

    fut.subscribe().block();
    Ok(fut
        .get()
        .expect("should be ready")
        .expect("should be first call")?)
}

impl Guest for Component {
    fn handle(_request: IncomingRequest, response_out: ResponseOutparam) {
        // First call the outgoing handler to send the request
        // We expect that the middleware will not modify the path
        let response = send(|req| {
            outgoing_handler::handle(req, None).context("failed to send to outgoing handler")
        })
        .expect("expected success");

        let body = response.consume().unwrap();
        let stream = body.stream().unwrap();
        let mut response = Vec::new();
        while let Ok(bytes) = stream.blocking_read(u64::MAX) {
            response.extend(bytes);
        }

        // Assert that middleware is not in the response
        let text = std::str::from_utf8(&response).unwrap();
        assert!(!text.contains("middleware"));
        drop(stream);
        drop(body);
        drop(response);

        // Send the request again through the middleware and
        // splice the response into the outgoing response body
        let response = send(|req| {
            middleware::exec::exec(req, None).context("failed to send to outgoing handler")
        })
        .expect("expected success");

        let outgoing = OutgoingResponse::new(response.headers());
        outgoing.set_status_code(response.status()).unwrap();

        let body = outgoing.body().unwrap();
        let response_body = response.consume().unwrap();
        let response_stream = response_body.stream().unwrap();

        ResponseOutparam::set(response_out, Ok(outgoing));

        let body_stream = body.write().unwrap();

        body_stream
            .blocking_splice(&response_stream, u64::MAX)
            .unwrap();

        drop(response_stream);
        drop(response_body);
        drop(body_stream);

        OutgoingBody::finish(body, None).unwrap();
    }
}
