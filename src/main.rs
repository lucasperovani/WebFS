use std::net::SocketAddr;

use tower_http::services::{ServeDir, ServeFile};
use axum::{
	routing::{delete, get, put},
	Router
};
use colored::Colorize;

mod routes;

#[tokio::main]
async fn main() {
	// Set the data directory
	let data_dir = match std::env::var("DATA_DIR") {
		Ok(data_dir) => data_dir,
		Err(_) => {
			eprintln!(
				"\n{}\n{}\n",
				"❌ DATA_DIR environment variable not set!".bold().red(),
				"Please set the DATA_DIR environment variable to the path \
				of the data directory!"
			);
			return;
		}
	};

	// Set Socket Address
	let port: u16 = match std::env::var("PORT") {
		Ok(port) => port.parse().unwrap_or(3000),
		Err(_) => 3000,
	};
	let address = SocketAddr::from(([0, 0, 0, 0], port));
	
	// Build application
	let app = Router::new()
		.route_service("/", ServeFile::new("assets/html/index.html"))
		.nest_service("/js", ServeDir::new("assets/js"))
		.nest_service("/css", ServeDir::new("assets/css"))
		.nest_service("/bootstrap", ServeDir::new("assets/bootstrap"))
		.nest_service("/fontawesome", ServeDir::new("assets/fontawesome"))
		.nest_service("/jquery", ServeDir::new("assets/jquery"))
		.route("/api/v1/mkdir", put(routes::mk::create_dir))
		.route("/api/v1/rmdir", delete(routes::rm::delete_dir))
		.route("/api/v1/ls", get(routes::ls::ls_dir))
		.route("/api/v1/mv", put(routes::mv::move_dir_or_file))
		.route("/api/v1/download", get(routes::transfer::download_file))
		.route("/api/v1/rm", delete(routes::rm::delete_file))
		.route("/api/v1/upload", put(routes::transfer::upload_file))
		.route("/api/v1/cp", put(routes::cp::copy))
		.with_state(routes::types::ServerState {data_dir});

	// Create the listener
	let listener = match tokio::net::TcpListener::bind(
		address
	).await {
		Ok(listener) => listener,
		Err(error) => {
			eprintln!(
				"{} {}", "❌ Failed to bind to port 3000:".bold().red(), error
			);
			return;
		}
	};

	
	println!("\n{}", "🔧 Preparing the server...".bold().blue());
	let url = format!(
		"http://{}", listener.local_addr().unwrap_or(address)
	);

	// Start the server
	let server = axum::serve(listener, app)
		.with_graceful_shutdown(routes::shutdown::shutdown_signal());

	println!("{} {}", "🚀 Listening on:".bold().blue(), url.bold().green());

	// Wait for the server to stop
	match server.await {
		Ok(_) => println!("{}", "🛑 Server stopped".bold().red()),
		Err(error) => eprintln!(
			"{} {}", "❌ Server error:".bold().red(), error
		),
	};
}
