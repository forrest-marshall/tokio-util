//! Spawn simple cross-task services.
//!
//! Use the `spawn` function to launch a service into the tokio event-loop, returning
//! a cloneable handle.
//!
//! ## Example
//!
//! Use `spawn` and a `HashMap` to create a shared key-value store, with `Get`, `Set`
//! and `Del` operations modeled by an enum:
//!
//! ```
//! extern crate tokio_util;
//! extern crate tokio;
//!
//! use std::collections::HashMap;
//! use tokio_util::service;
//! use tokio::prelude::*; 
//! 
//!
//! #[derive(Debug)]
//! enum Op<K,V> {
//!     Get(K),
//!     Set(K,V),
//!     Del(K),
//! }
//!
//!
//! # fn main() {
//! // `spawn` must be called within an event-loop, so we
//! // wrap out setup logic in a closure to defer execution.
//! let spawn_map = || {
//!     let mut map = HashMap::new();
//!     let handle = service::spawn(move |op| {
//!         match op {
//!             Op::Get(key) => Ok(map.get(key).cloned()),
//!             Op::Set(key,val) => Ok(map.insert(key,val)),
//!             Op::Del(key) => Ok(map.remove(key)),
//!         }
//!     });
//!     Ok(handle)
//! };
//!
//! // Lazily spawn the map, and then fire off some requests.
//! let make_requests = future::lazy(spawn_map).and_then(|handle| {
//!
//!     let requests = vec![
//!         handle.call(Op::Set("hello","world")),
//!         handle.call(Op::Get("hello")),
//!         handle.call(Op::Del("hello")),
//!         handle.call(Op::Get("hello")),
//!     ];
//!
//!     future::collect(requests).then(|result| {
//!         assert_eq!(result.unwrap(),vec![None,Some("world"),Some("world"),None]);
//!         Ok(())
//!     })
//! });
//!
//! tokio::run(make_requests);
//! # }
//!
//! ```
//!
use tokio_channel::{oneshot,mpsc};
use tokio::prelude::*;
use tokio;
use std::{fmt,error};


/// Spawn a service to the event-loop.
///
/// Spawns a threadsafe service to the event-loop, returning a
/// cloneable/sendable handle. See module-level docs for example usage.
///
/// ## panics
///
/// This function will panic if called outside of an event-loop.
///
pub fn spawn<Srv,Req,Rsp>(mut service: Srv) -> Handle<Req,Rsp>
        where Srv: Service<Req,Rsp,Error=()> + Send + 'static, Srv::Future: Send + 'static,
              Req: Send + 'static, Rsp: Send + 'static {
    let (tx,rx) = mpsc::unbounded();
    let handle = Handle::new(tx);
    let serve = move |call: Call<_,_>| {
        let Call { req, tx } = call;
        let work = service.call(req).and_then(move |rsp| {
            let _ = tx.send(rsp);
            Ok(())
        });
        tokio::spawn(work);
    };

    let work = rx.map(serve).for_each(|()| Ok(()));
    tokio::spawn(work);
    handle
}


/// An asynchronous service.
///
pub trait Service<Req,Rsp> {

    /// Error raised if service fails
    type Error;

    /// Future which drives service's work
    type Future: Future<Item=Rsp,Error=Self::Error>;

    /// Execute a call against this service
    fn call(&mut self, req: Req) -> Self::Future;
}


impl<F,T,Req> Service<Req,T::Item> for F where F: FnMut(Req) -> T, T: IntoFuture {

    type Error = T::Error;

    type Future = T::Future;

    fn call(&mut self, req: Req) -> Self::Future {
        (self)(req).into_future()
    }
}


#[derive(Debug)]
struct Call<Req,Rsp> {
    req: Req,
    tx: oneshot::Sender<Rsp>,
}


/// Cloneable handle to a spawned service.
/// 
/// Allows one or more tasks to asynchronously call a service.
/// See module-level docs for example usage.
///
/// *NOTE*: Services are assumed to be inaccessable and terminated at
/// the point when the last copy of their handle is dropped.
///
#[derive(Debug)]
pub struct Handle<Req,Rsp> {
    inner: mpsc::UnboundedSender<Call<Req,Rsp>>,
}


impl<Req,Rsp> Clone for Handle<Req,Rsp> {

    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}


impl<Req,Rsp> Handle<Req,Rsp> {

    fn new(inner: mpsc::UnboundedSender<Call<Req,Rsp>>) -> Self { Self { inner } }

    /// Execute a call against the associated service
    pub fn call(&self, req: Req) -> impl Future<Item=Rsp,Error=Error<Req>> {
        let (tx,rx) = oneshot::channel();
        let call = Call { req, tx };
        self.inner.unbounded_send(call).into_future().from_err()
            .and_then(move |()| rx.from_err())
    }
}


/// Error raised by a service handle
#[derive(Debug,Copy,Clone)]
pub enum Error<T> {
    /// Failed to send request; service has failed.
    SendError(T),
    /// Response channel was cancelled; request has failed.
    Canceled,
}


impl<T> Error<T> {

    fn as_str(&self) -> &str {
        match self {
            Error::SendError(_) => "unable to send request (receiver dropped)",
            Error::Canceled => "request cancelled (rsp channel dropped)",
        }
    }
}


impl<T> fmt::Display for Error<T> where T: fmt::Debug {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}


impl<T> error::Error for Error<T> where T: fmt::Debug {

    fn description(&self) -> &str { self.as_str() }
}


impl<Req,Rsp> From<mpsc::SendError<Call<Req,Rsp>>> for Error<Req> {

    fn from(err: mpsc::SendError<Call<Req,Rsp>>) -> Self {
        let Call { req, .. } = err.into_inner();
        Error::SendError(req)
    }
}


impl<T> From<oneshot::Canceled> for Error<T> {

    fn from(_: oneshot::Canceled) -> Self { Error::Canceled }
}


