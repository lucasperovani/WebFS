use std::{net::SocketAddr, path::Path};

use path_clean::PathClean;
use tokio::signal;
use tokio_util::io::ReaderStream;
use tower_http::services::ServeFile;
use axum::{
	body::Body,
	extract::{Query, State},
	http::{header, StatusCode},
	response::IntoResponse,
	routing::{delete, get, put},
	Json,
	Router
};
use serde::{Deserialize, Serialize};
use colored::Colorize;

#[tokio::main]
async fn main() {
	// Set the data directory
	let data_dir =
		"\\\\?\\C:\\Users\\lucas\\OneDrive\\Documents\\Projetos".to_string();

	// Set Socket Address
	let address = SocketAddr::from(([127, 0, 0, 1], 3000));
	
	// Build application
	let app = Router::new()
		.route_service("/", ServeFile::new("assets/index.html"))
		.route("/mkdir", put(create_dir))
		.route("/rmdir", delete(delete_dir))
		.route("/ls", get(ls_dir))
		.route("/mv", put(move_dir_or_file))
		.route("/download", get(download_file))
		.route("/rm", delete(delete_file))
		.with_state(ServerState {data_dir});

	// Create the listener
	let listener = match tokio::net::TcpListener::bind(
		"127.0.0.1:3000"
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
		.with_graceful_shutdown(shutdown_signal());

	println!("{} {}", "🚀 Listening on:".bold().blue(), url.bold().green());

	// Wait for the server to stop
	match server.await {
		Ok(_) => println!("{}", "🛑 Server stopped".bold().red()),
		Err(error) => eprintln!(
			"{} {}", "❌ Server error:".bold().red(), error
		),
	};
}

async fn shutdown_signal() {
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

#[derive(Clone)]
struct ServerState {
	data_dir: String,
}

#[derive(Serialize)]
struct File {
	name: String,
	size: u64,
	is_dir: bool,
}

#[derive(Serialize)]
struct LsDirectoryResponse {
	success: bool,
	message: String,
	files: Vec<File>
}

#[derive(Deserialize)]
struct DirectoryQuery {
	path: String,
}

#[derive(Deserialize)]
struct MoveQuery {
	from: String,
	to: String,
}

#[derive(Serialize)]
struct SimpleServerResponse {
	success: bool,
	message: String,
}

/// List the files in a directory
async fn ls_dir(
	query_params: Query<DirectoryQuery>,
	State(state): State<ServerState>,
) -> (StatusCode, Json<LsDirectoryResponse>) {
	let path = Path::new(&state.data_dir)
		.join(query_params.path.clone())
		.clean();

	// Confirm that the path is part of the data directory
	if !path.starts_with(&state.data_dir) {
		return (
			StatusCode::BAD_REQUEST,
			Json(LsDirectoryResponse {
				success: false,
				message: "Invalid path".to_string(),
				files: vec![]
			})
		);
	}

	if !path.try_exists().unwrap_or(false) {
		return (
			StatusCode::NOT_FOUND,
			Json(LsDirectoryResponse {
				success: false,
				message: "Path not found".to_string(),
				files: vec![]
			})
		);
	}

	if !path.is_dir() {
		return (
			StatusCode::BAD_REQUEST,
			Json(LsDirectoryResponse {
				success: false,
				message: "Path is not a directory".to_string(),
				files: vec![]
			})
		);
	}

	let directory_entries = match path.read_dir() {
		Ok(entries) => entries,
		Err(err) => return (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(LsDirectoryResponse {
				success: false,
				message: format!("Failed to read directory: {}", err),
				files: vec![]
			})
		),
	};

	let mut entries = vec![];
	for entry in directory_entries {
		let entry = match entry {
			Ok(entry) => entry,
			Err(error) => return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(LsDirectoryResponse {
					success: false,
					message: format!(
						"Failed to read directory entry: {}",
						error
					),
					files: vec![]
				})
			),
		};

		let metadata = match entry.metadata() {
			Ok(metadata) => metadata,
			Err(error) => return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(LsDirectoryResponse {
					success: false,
					message: format!("Failed to read metadata: {}", error),
					files: vec![]
				})
			),
		};

		let file = File {
			name: entry
				.file_name()
				.into_string()
				.unwrap_or("unknown".to_string()),
			size: metadata.len(),
			is_dir: metadata.is_dir(),
		};

		entries.push(file);
	}
	

	return (
		StatusCode::OK,
		Json(LsDirectoryResponse {
			success: true,
			message: "Directory listed successfully".to_string(),
			files: entries
		})
	);
}

/// Create directory
async fn create_dir(
	query_params: Query<DirectoryQuery>,
	State(state): State<ServerState>,
) -> (StatusCode, Json<SimpleServerResponse>) {
	let path = Path::new(&state.data_dir)
		.join(query_params.path.clone())
		.clean();

	// Confirm that the path is part of the data directory
	if !path.starts_with(&state.data_dir) {
		return (
			StatusCode::BAD_REQUEST,
			Json(SimpleServerResponse {
				success: false,
				message: "Invalid path".to_string(),
			})
		);
	}

	if path.try_exists().unwrap_or(true) {
		return (
			StatusCode::BAD_REQUEST,
			Json(SimpleServerResponse {
				success: false,
				message: "Directory already exists".to_string(),
			})
		);
	}

	match tokio::fs::create_dir(path.clone()).await {
		Ok(_) => (
			StatusCode::OK,
			Json(SimpleServerResponse {
				success: true,
				message: "Directory created successfully".to_string(),
			})
		),
		Err(err) => (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(SimpleServerResponse {
				success: false,
				message: format!("Failed to create directory: {}", err),
			})
		),
	}
}

/// Delete whole directory
async fn delete_dir(
	query_params: Query<DirectoryQuery>,
	State(state): State<ServerState>,
) -> (StatusCode, Json<SimpleServerResponse>) {
	let path = Path::new(&state.data_dir)
		.join(query_params.path.clone())
		.clean();

	// Confirm that the path is part of the data directory
	if !path.starts_with(&state.data_dir) {
		return (
			StatusCode::BAD_REQUEST,
			Json(SimpleServerResponse {
				success: false,
				message: "Invalid path".to_string(),
			})
		);
	}

	if !path.try_exists().unwrap_or(false) || !path.is_dir() {
		return (
			StatusCode::NOT_FOUND,
			Json(SimpleServerResponse {
				success: false,
				message: "Directory not found".to_string(),
			})
		);
	}

	match tokio::fs::remove_dir_all(path.clone()).await {
		Ok(_) => (
			StatusCode::OK,
			Json(SimpleServerResponse {
				success: true,
				message: "Directory deleted successfully".to_string(),
			})
		),
		Err(err) => (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(SimpleServerResponse {
				success: false,
				message: format!("Failed to delete directory: {}", err),
			})
		),
	}
}

/// Move directory
async fn move_dir_or_file(
	query_params: Query<MoveQuery>,
	State(state): State<ServerState>,
) -> (StatusCode, Json<SimpleServerResponse>) {
	let from = Path::new(&state.data_dir)
		.join(query_params.from.clone())
		.clean();

	let to = Path::new(&state.data_dir)
		.join(query_params.to.clone())
		.clean();

	// Confirm that the path is part of the data directory
	if
		!from.starts_with(&state.data_dir) ||
		!to.starts_with(&state.data_dir)
	{
		return (
			StatusCode::BAD_REQUEST,
			Json(SimpleServerResponse {
				success: false,
				message: "Invalid paths".to_string(),
			})
		);
	}

	// If the source directory does not exist return error
	if !from.try_exists().unwrap_or(false) {
		return (
			StatusCode::NOT_FOUND,
			Json(SimpleServerResponse {
				success: false,
				message: "Source path not found".to_string(),
			})
		);
	}

	// If the destination directory does exist return error
	if to.try_exists().unwrap_or(true) {
		return (
			StatusCode::BAD_REQUEST,
			Json(SimpleServerResponse {
				success: false,
				message: "Destination path already exists".to_string(),
			})
		);
	}

	match tokio::fs::rename(from.clone(), to.clone()).await {
		Ok(_) => (
			StatusCode::OK,
			Json(SimpleServerResponse {
				success: true,
				message: "Directory moved successfully".to_string(),
			})
		),
		Err(err) => (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(SimpleServerResponse {
				success: false,
				message: format!("Failed to move directory: {}", err),
			})
		),
	}
}

/// Return the file to download
async fn download_file(
	query_params: Query<DirectoryQuery>,
	State(state): State<ServerState>,
) -> impl IntoResponse {
	let path = Path::new(&state.data_dir)
		.join(query_params.path.clone())
		.clean();

	// Confirm that the path is part of the data directory
	if !path.starts_with(&state.data_dir) {
		return Err((
			StatusCode::BAD_REQUEST,
			"Invalid path".to_string()
		));
	}

	if !path.try_exists().unwrap_or(false) || !path.is_file() {
		return Err((
			StatusCode::NOT_FOUND,
			"File not found".to_string()
		));
	}
	
	// `File` implements `AsyncRead`
	let file = match tokio::fs::File::open(path.clone()).await {
		Ok(file) => file,
		Err(err) => return Err((
			StatusCode::NOT_FOUND,
			format!("File not found: {}", err)
		)),
	};

	// convert the `AsyncRead` into a `Stream`
	let stream = ReaderStream::new(file);
	// convert the `Stream` into an `axum::body::HttpBody`
	let body = Body::from_stream(stream);

	let headers = [(
		header::CONTENT_TYPE,
		format!("{:?}; charset=utf-8", mime_guess::from_path(&path).first()),
	),(
		header::CONTENT_DISPOSITION,
		format!(
			"attachment; filename={:?}",
			path.file_name().unwrap_or(std::ffi::OsStr::new("unknown"))
		),
	)];

	Ok((headers, body).into_response())
}

/// Remove a file
async fn delete_file(
	query_params: Query<DirectoryQuery>,
	State(state): State<ServerState>,
) -> (StatusCode, Json<SimpleServerResponse>) {
	let path = Path::new(&state.data_dir)
		.join(query_params.path.clone())
		.clean();

	// Confirm that the path is part of the data directory
	if !path.starts_with(&state.data_dir) {
		return (
			StatusCode::BAD_REQUEST,
			Json(SimpleServerResponse {
				success: false,
				message: "Invalid path".to_string(),
			})
		);
	}

	if !path.try_exists().unwrap_or(false) || !path.is_file() {
		return (
			StatusCode::NOT_FOUND,
			Json(SimpleServerResponse {
				success: false,
				message: "File not found".to_string(),
			})
		);
	}

	match tokio::fs::remove_file(path.clone()).await {
		Ok(_) => (
			StatusCode::OK,
			Json(SimpleServerResponse {
				success: true,
				message: "File deleted successfully".to_string(),
			})
		),
		Err(err) => (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(SimpleServerResponse {
				success: false,
				message: format!("Failed to delete file: {}", err),
			})
		),
	}
}
