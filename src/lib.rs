extern crate iron;
extern crate url;

use iron::BeforeMiddleware;
use iron::headers::ContentLength;
use iron::prelude::*;
use iron::status;

use std::default::Default;
use std::error::Error;
use std::fmt;
use std::io::Read;

const MAX_BODY_DEFAULT: u64 = 5e6 as u64;
const MAX_URL_DEFAULT: usize = 256;

/// The error thrown when the request body size is larger than the limit set
/// by the middleware.
#[derive(Debug)]
pub struct RequestTooLarge;

/// `RequestLimit` configures the maximum payload size and URL length for Iron's middleware system.
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

impl Default for RequestLimit {
    fn default() -> RequestLimit {
        RequestLimit {
            max_body: MAX_BODY_DEFAULT,
            max_url_length: MAX_URL_DEFAULT,
        }
    }
}

impl RequestLimit {
    /// set_max_body_size overrides the maximum body request size of 5 MB.
    pub fn set_max_body_size(&mut self, max_body: u64) {
        self.max_body = max_body;
    }

    /// set_max_url_length overrides the default maximum URL length of 256.
    pub fn set_max_url_length(&mut self, max_url_length: usize) {
        self.max_url_length = max_url_length;
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
        let real_url: url::Url = u.into();

        real_url.as_str().len() <= self.max_url_length
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

    static GOOGLE: &'static str = "https://google.com";

    #[test]
    fn check_ok_response() {
        let b = RequestLimit::default();
        assert!(b.check_payload(5).is_ok());
    }

    #[test]
    fn check_err_response() {
        let mut b = RequestLimit::default();
        b.set_max_body_size(1);
        assert!(b.check_payload(2).is_err());
    }

    #[test]
    fn test_lengthy_url() {
        let mut b = RequestLimit::default();
        b.set_max_url_length(5);
        assert_eq!(false, b.check_url_length(::iron::Url::parse(GOOGLE).unwrap()));
    }

    #[test]
    fn test_valid_url() {
        let b = RequestLimit::default();
        assert!(b.check_url_length(::iron::Url::parse(GOOGLE).unwrap()));
    }
}
