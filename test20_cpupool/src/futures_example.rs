/*

extern crate futures;

use std::io;
use std::time::Duration;
use futures::prelude::*;
use futures::future::Map;

// A future is actually a trait implementation, so we can generically take a
// future of any integer and return back a future that will resolve to that
// value plus 10 more.
//
// Note here that like iterators, we're returning the `Map` combinator in
// the futures crate, not a boxed abstraction. This is a zero-cost
// construction of a future.
fn add_ten<F>(future: F) -> Map<F, fn(i32) -> i32>
    where F: Future<Item=i32>,
{
    fn add(a: i32) -> i32 { 
        a + 10 
    }
    future.map(add)
}

// Not only can we modify one future, but we can even compose them together!
// Here we have a function which takes two futures as input, and returns a
// future that will calculate the sum of their two values.
//
// Above we saw a direct return value of the `Map` combinator, but
// performance isn't always critical and sometimes it's more ergonomic to
// return a trait object like we do here. Note though that there's only one
// allocation here, not any for the intermediate futures.
fn add<'a, A, B>(a: A, b: B) -> Box<Future<Item=i32, Error=A::Error> + 'a>
    where A: Future<Item=i32> + 'a,
          B: Future<Item=i32, Error=A::Error> + 'a,
{
    Box::new(a.join(b).map(|(a, b)| a + b))
}

// Futures also allow chaining computations together, starting another after
// the previous finishes. Here we wait for the first computation to finish,
// and then decide what to do depending on the result.
fn download_timeout(url: &str, timeout_dur: Duration) -> Box<Future<Item=Vec<u8>, Error=io::Error>> {
    use std::io;
    use std::net::{SocketAddr, TcpStream};

    type IoFuture<T> = Box<Future<Item=T, Error=io::Error>>;

    // First thing to do is we need to resolve our URL to an address. This
    // will likely perform a DNS lookup which may take some time.
    let addr = resolve(url);

    // After we acquire the address, we next want to open up a TCP
    // connection.
    let tcp = addr.and_then(|addr| {
        connect(&addr)
    });

    // After the TCP connection is established and ready to go, we're off to
    // the races!
    let data = tcp.and_then(|conn| {
        download(conn)
    });

    // That all might take awhile, though, so let's not wait too long for it
    // to all come back. The `select` combinator here returns a future which
    // resolves to the first value that's ready plus the next future.
    //
    // Note we can also use the `then` combinator which is similar to
    // `and_then` above except that it receives the result of the
    // computation, not just the successful value.
    //
    // Again note that all the above calls to `and_then` and the below calls
    // to `map` and such require no allocations. We only ever allocate once
    // we hit the `Box::new()` call at the end here, which means we've built
    // up a relatively involved computation with only one box, and even that
    // was optional!

    let data = data.map(Ok);
    let timeout = timeout(timeout_dur).map(Err);

    let ret = data.select(timeout).then(|result| {
        match result {
            // One future succeeded, and it was the one which was
            // downloading data from the connection.
            Ok((Ok(data), _other_future)) => Ok(data),

            // The timeout fired, and otherwise no error was found, so
            // we translate this to an error.
            Ok((Err(_timeout), _other_future)) => {
                Err(io::Error::new(io::ErrorKind::Other, "timeout"))
            }

            // A normal I/O error happened, so we pass that on through.
            Err((e, _other_future)) => Err(e),
        }
    });
    return Box::new(ret);

    fn resolve(url: &str) -> IoFuture<SocketAddr> {
    }

    fn connect(hostname: &SocketAddr) -> IoFuture<TcpStream> {
    }

    fn download(stream: TcpStream) -> IoFuture<Vec<u8>> {
    }

    fn timeout(stream: Duration) -> IoFuture<()> {
    }
}*/

pub fn test_futures(){

}