extern crate iron;
extern crate url;

use iron::BeforeMiddleware;
use iron::headers::ContentLength;
use iron::prelude::*;
use iron::status;

use std::error::Error;
use std::fmt;
use std::io::Read;

/// The error thrown when the request body size is larger than the limit set
/// by the middleware.
#[derive(Debug)]
pub struct RequestTooLarge;

pub struct RequestLimit {
    /// The maximum size of a payload.
    max_body: u64,

    /// The maximum size of a URL.
    max_url_length: usize,
}

impl fmt::Display for RequestTooLarge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl Error for RequestTooLarge {
    fn description(&self) -> &str {
        "Request too large"
    }
}

impl RequestLimit {
    /// Construct a new request size limiter for Iron.
    pub fn new(max_body: u64, max_url_length: usize) -> RequestLimit {
        RequestLimit {
            max_body: max_body,
            max_url_length: max_url_length,
        }
    }

    // Set the proper error response if the request body is too big or carry on
    // down the chain.
    fn check_payload(&self, total: u64) -> IronResult<()> {
        if total > self.max_body {
            Err(IronError::new(RequestTooLarge, status::PayloadTooLarge))
        } else {
            Ok(())
        }
    }

    // Ensure that the URL length doesn't exceed the maximum.
    fn check_url_length(&self, u: iron::Url) -> bool {
        u.into_generic_url().as_str().len() <= self.max_url_length
    }
}

impl BeforeMiddleware for RequestLimit {
    // This middleware tries to read the content length of the request first
    // before reading the request body.
    fn before(&self, req: &mut Request) -> IronResult<()> {
        if !self.check_url_length(req.url.clone()) {
            return Err(IronError::new(RequestTooLarge, status::PayloadTooLarge));
        }

        match req.headers.get::<ContentLength>() {
            Some(l) => { self.check_payload(l.0) },
            None => { self.check_payload(req.body.by_ref().bytes().count() as u64) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RequestLimit;

    static URL_MAX: usize = 256;

    #[test]
    fn check_ok_response() {
        let b = RequestLimit::new(5, URL_MAX);
        assert!(b.check_payload(5).is_ok());
    }

    #[test]
    fn check_err_response() {
        let b = RequestLimit::new(1, URL_MAX);
        assert!(b.check_payload(2).is_err());
    }
}
