# limiter

[![Build Status](https://travis-ci.org/gsquire/limiter.svg?branch=master)](https://travis-ci.org/gsquire/limiter)

This is an example of `BeforeMiddleware` for the [Iron](https://github.com/iron/iron) framework.  It limits the request body size
by checking first the Content-Length header, then the size of the payload. The response is either
an HTTP 413 or it continues down the chain.

Include this in your `Cargo.toml` file:

```sh
[dependencies]
limiter = "0.1"
```

### Example

```rust
extern crate iron;
extern crate limiter;

use iron::prelude::*;

use limiter::BodyLimit;

fn index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Test")))
}

fn main() {
    let max_request = BodyLimit::new(5000);
    let mut chain = Chain::new(index);
    chain.link_before(max_request);
    Iron::new(chain).http("localhost:3000").unwrap();
}
```

### License
MIT
