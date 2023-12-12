cargo_component_bindings::generate!();

use bindings::{
    exports::wasi::http::incoming_handler::Guest,
    wasi::http::{
        outgoing_handler::{handle, OutgoingRequest},
        types::{
            Headers, IncomingRequest, OutgoingBody, OutgoingResponse, ResponseOutparam, Scheme,
        },
    },
};

struct Component;

impl Guest for Component {
    fn handle(_request: IncomingRequest, response_out: ResponseOutparam) {
        let req = OutgoingRequest::new(Headers::new());
        req.set_scheme(Some(&Scheme::Https)).unwrap();
        req.set_authority(Some("echo.free.beeceptor.com")).unwrap();
        req.set_path_with_query(Some("/")).unwrap();
        let outgoing_body = req.body().unwrap();

        let fut = handle(req, None).unwrap();
        OutgoingBody::finish(outgoing_body, None).unwrap();

        fut.subscribe().block();
        let resp = fut
            .get()
            .expect("should be ready")
            .unwrap()
            .expect("expected success");

        let out_resp = OutgoingResponse::new(resp.headers());
        out_resp.set_status_code(resp.status()).unwrap();
        let body = out_resp.body().unwrap();

        let incoming_body = resp.consume().unwrap();
        let incoming_stream = incoming_body.stream().unwrap();

        ResponseOutparam::set(response_out, Ok(out_resp));

        let output_stream = body.write().unwrap();

        output_stream
            .blocking_splice(&incoming_stream, u64::MAX)
            .unwrap();

        drop(incoming_stream);
        drop(incoming_body);

        drop(output_stream);
        OutgoingBody::finish(body, None).unwrap();
    }
}
