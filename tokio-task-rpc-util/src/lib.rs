//! Library code for use with tokio-task-rpc
//!
//! tokio-task-rpc generated code that uses types from this crate. Any user
//! needs to add both crates to their dependencies.

/// based on: <https://www.reddit.com/r/rust/comments/jpg0pp/comment/gbeusao/?utm_source=share&utm_medium=web2x&context=3>
pub trait FnHelper<'a, P, O> {
    type R: core::future::Future<Output = O> + 'a;

    fn call(&self, val: &'a mut P) -> Self::R;
}

impl<'a, R, F, P, O> FnHelper<'a, P, O> for F
where
    R: core::future::Future<Output = O> + 'a,
    F: Fn(&'a mut P) -> R,
    P: 'a,
{
    type R = R;

    fn call(&self, val: &'a mut P) -> Self::R {
        (self)(val)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("request receiver closed")]
    ReceiverClosed,

    #[error("failed to read return value")]
    RecvError,
}
