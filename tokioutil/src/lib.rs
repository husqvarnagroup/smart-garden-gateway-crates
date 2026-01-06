//! Provide spawn_named

// Source: https://github.com/tokio-rs/console/blob/9a50b63082d7cebc5cba408139bcf12348e9466a/console-subscriber/src/lib.rs#L973
//
// See LICENSE.tokio for licensing information.
#[track_caller]
pub fn spawn_named<T>(
    _name: &str,
    task: impl std::future::Future<Output = T> + Send + 'static,
) -> tokio::task::JoinHandle<T>
where
    T: Send + 'static,
{
    #[cfg(tokio_unstable)]
    return tokio::task::Builder::new().name(_name).spawn(task);

    #[cfg(not(tokio_unstable))]
    tokio::spawn(task)
}
