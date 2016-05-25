extern crate iron;

use iron::BeforeMiddleware;
use iron::headers::ContentLength;
use iron::prelude::*;
use iron::status;

use std::error::Error;
use std::fmt;
use std::io::Read;

// The default is 20 MB.
const DEFAULT_LIMIT: u64 = 2e7 as u64;

/// The error thrown when the request body size is larger than the limit set
/// by the middleware.
#[derive(Debug)]
pub struct RequestTooLarge;

pub struct BodyLimit {
    max_body: u64,
}

impl fmt::Display for RequestTooLarge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Request body too large.")
    }
}

impl Error for RequestTooLarge {
    fn description(&self) -> &str {
        "Request too large"
    }
}

impl BodyLimit {
    /// Construct a new request body size limiter for Iron.
    pub fn new(max: u64) -> BodyLimit {
        if max <= 0 {
            return BodyLimit { max_body: DEFAULT_LIMIT };
        }
        BodyLimit { max_body: max }
    }

    // Set the proper error response if the request body is too big or carry on
    // down the chain.
    fn check_octets(&self, total: u64) -> IronResult<()> {
        if total > self.max_body {
            Err(IronError::new(RequestTooLarge, status::PayloadTooLarge))
        } else {
            Ok(())
        }
    }
}

impl BeforeMiddleware for BodyLimit {
    // This middleware tries to read the content length of the request first
    // before reading the request body.
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let length = req.headers.get::<ContentLength>();
        match length {
            Some(l) => { self.check_octets(l.0) },
            None => { self.check_octets(req.body.by_ref().bytes().count() as u64) }
        }
    }
}
