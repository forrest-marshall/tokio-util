use tokio::prelude::*;
use std::{fmt,error};

/// I swear, this never happens.
///
#[derive(Hash,Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub enum Never { }



/// Implement `From<Never>` against a locally defined type.
/// 
/// This is especially helpful for dealing with APIs such as `Result<..,Never>`
/// or `Future<..,Error=Never>`, as it allows reliance on helpers like the `?`
/// operator, and `Future::from_err`.  Prefer using this over `unwrap` and similar
/// when dealing with `Never` errors.
///
/// ## Example
///
/// ```
/// #[macro_use]
/// extern crate tokio_util;
/// 
/// use tokio_util::Never;
///
///
/// #[derive(Debug)]
/// pub struct MyError;
///
/// from_never!(MyError);
/// 
///
/// fn never_fails() -> Result<(),Never> {
///     // Do an operation which is infallible but must return a 
///     // `Result` for some reason...
///     Ok(())
/// }
/// 
///
/// fn do_stuff() -> Result<(),MyError> {
///     never_fails()?;
///     // other work which might actually fail...
///     Ok(())
/// }
///
/// # fn main() {
/// #     do_stuff().unwrap();
/// # }
/// ```
///
#[macro_export]
macro_rules! from_never {
    ($type:ty) => {
        impl From<$crate::Never> for $type {

            fn from(never: $crate::Never) -> Self { match never { } }
        }
    }
}


impl Never {

    pub fn into<T>(self) -> T { match self { } }
}


impl Future for Never {

    type Item = Never;

    type Error = Never;

    fn poll(&mut self) -> Poll<Self::Item,Self::Error> { match *self { } }
}


impl Stream for Never {

    type Item = Never;

    type Error = Never;

    fn poll(&mut self) -> Poll<Option<Self::Item>,Self::Error> { match *self { } }
}


impl<T> AsRef<T> for Never {

    fn as_ref(&self) -> &T { match *self { } }
}

impl<T> AsMut<T> for Never {

    fn as_mut(&mut self) -> &mut T { match *self { } } 
}


impl fmt::Display for Never {

    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result { match *self { } }
}


impl error::Error for Never {

    fn description(&self) -> &str { match *self { } }

    fn cause(&self) -> Option<&dyn error::Error> { match *self { } }
}

