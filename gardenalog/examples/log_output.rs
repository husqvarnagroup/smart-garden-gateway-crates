
fn span_example() {
    let _span = tracing::info_span!("in_span").entered();
    tracing::error!("error in span");
    tracing::info!("error in span");
    tracing::info!("info in span");
    tracing::debug!("debug in span");
    tracing::trace!("trace in span");
}

fn main() {
    gardenalog::init_tracing();
    tracing::error!("error before span");
    tracing::info!("error before span");
    tracing::info!("info before span");
    tracing::debug!("debug before span");
    tracing::trace!("trace before span");

    span_example();

    tracing::error!("error after span");
    tracing::info!("error after span");
    tracing::info!("info after span");
    tracing::debug!("debug after span");
    tracing::trace!("trace after span");
}
