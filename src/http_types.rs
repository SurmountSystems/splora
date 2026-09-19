// SPDX-License-Identifier: Unlicense
//! Shared hyper 1 body types for REST, queue unix, and `splora-http`.

use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Full};
use hyper_util::rt::TokioTimer;
use std::convert::Infallible;
use std::time::Duration;

/// Boxed response body used by REST, queue unix, and the TLS front.
pub type HttpBody = http_body_util::combinators::BoxBody<Bytes, Infallible>;
/// Incoming request on hyper 1 servers.
pub type HttpRequest = hyper::Request<hyper::body::Incoming>;
/// Response with [`HttpBody`].
pub type HttpResponse = hyper::Response<HttpBody>;

pub fn full_body(data: impl Into<Bytes>) -> HttpBody {
    Full::new(data.into()).boxed()
}

pub fn empty_body() -> HttpBody {
    Empty::<Bytes>::new().boxed()
}

/// HTTP/1.1 builder with a Timer (required) and a header-read timeout.
/// Default in hyper 1 is 30 seconds when a timer is set. Callers pass 10 seconds.
pub fn http1_builder_with_header_read_timeout(
    timeout: Duration,
) -> hyper::server::conn::http1::Builder {
    let mut builder = hyper::server::conn::http1::Builder::new();
    builder
        .timer(TokioTimer::new())
        .header_read_timeout(timeout);
    builder
}
