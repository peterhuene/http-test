cargo_component_bindings::generate!();

use bindings::{
    exports::wasi::http::outgoing_handler::Guest,
    wasi::http::outgoing_handler::{
        self, ErrorCode, FutureIncomingResponse, OutgoingRequest, RequestOptions,
    },
};

struct Component;

impl Guest for Component {
    fn handle(
        request: OutgoingRequest,
        options: Option<RequestOptions>,
    ) -> Result<FutureIncomingResponse, ErrorCode> {
        request.set_path_with_query(Some("/middleware")).unwrap();
        outgoing_handler::handle(request, options)
    }
}
