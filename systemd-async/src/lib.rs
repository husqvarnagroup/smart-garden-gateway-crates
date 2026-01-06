//! This crate provides async implementations of functions you would find in
//! libsystemd.

/// notify the service manager about state changes.
///
/// See the man page for `sd_notify` for more information.
pub async fn notify(unset_env: bool, state: &str) -> std::io::Result<()> {
    let socket_path = match std::env::var_os("NOTIFY_SOCKET") {
        Some(path) => {
            log::debug!("notify socket: {:?}", path);
            path
        }
        None => {
            log::debug!("no notify socket found");
            return Ok(());
        }
    };
    if unset_env {
        std::env::remove_var("NOTIFY_SOCKET");
    }

    let sock = tokio::net::UnixDatagram::unbound()?;
    let len = sock.send_to(state.as_bytes(), socket_path).await?;
    if len != state.len() {
        Err(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "incomplete write",
        ))
    } else {
        Ok(())
    }
}
