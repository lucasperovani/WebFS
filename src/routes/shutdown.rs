use tokio::signal;

/// Waits for a shutdown signal from the OS.
///
/// This function returns when the OS sends a signal to stop the application.
/// The signals are either SIGINT (Ctrl+C) or SIGTERM. It is platform-specific.
/// This function implements the shutdown signal handling for Unix-like and
/// non Unix-like systems.
pub async fn shutdown_signal() {
	let ctrl_c = async {
		signal::ctrl_c()
			.await
			.expect("Failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		signal::unix::signal(signal::unix::SignalKind::terminate())
			.expect("Failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		_ = ctrl_c => {},
		_ = terminate => {},
	}
}