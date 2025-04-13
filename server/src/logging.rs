use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::{SubscriberInitExt, TryInitError};
use tracing_subscriber::{fmt, EnvFilter};

pub(crate) fn set_up_logging() -> Result<(), TryInitError> {
    // Parse an `EnvFilter` configuration from the `RUST_LOG` environment variable.
    let filter = EnvFilter::from_default_env();

    // Use the tracing subscriber `Registry`.
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .try_init()
}
