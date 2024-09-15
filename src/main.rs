use std::{io, net::SocketAddr, path::{Path, PathBuf}};

use path_clean::PathClean;
use tokio::{io::BufWriter, signal, fs::File};
use tokio_util::io::{ReaderStream, StreamReader};
use tower_http::services::{ServeDir, ServeFile};
use axum::{
	body::{Body, Bytes},
	extract::{Query, Request, State},
	http::{header, StatusCode},
	response::IntoResponse,
	routing::{delete, get, put},
	BoxError, Json, Router
};
use futures::{Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use colored::Colorize;

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
	let address = SocketAddr::from(([127, 0, 0, 1], 3000));
	
	// Build application
	let app = Router::new()
		.route_service("/", ServeFile::new("assets/html/index.html"))
		.nest_service("/js", ServeDir::new("assets/js"))
		.nest_service("/css", ServeDir::new("assets/css"))
		.nest_service("/bootstrap", ServeDir::new("assets/bootstrap"))
		.nest_service("/fontawesome", ServeDir::new("assets/fontawesome"))
		.nest_service("/jquery", ServeDir::new("assets/jquery"))
		.route("/api/v1/mkdir", put(create_dir))
		.route("/api/v1/rmdir", delete(delete_dir))
		.route("/api/v1/ls", get(ls_dir))
		.route("/api/v1/mv", put(move_dir_or_file))
		.route("/api/v1/download", get(download_file))
		.route("/api/v1/rm", delete(delete_file))
		.route("/api/v1/upload", put(upload_file))
		.route("/api/v1/cp", put(copy))
		.with_state(ServerState {data_dir});

	// Create the listener
	let listener = match tokio::net::TcpListener::bind(
		"0.0.0.0:3000"
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
struct FileInfo {
	name: String,
	size: u64,
	is_dir: bool,
}

#[derive(Serialize)]
struct LsDirectoryResponse {
	success: bool,
	message: String,
	files: Vec<FileInfo>
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
	State(state): State<ServerState>,
	query_params: Query<DirectoryQuery>,
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

	// If the path does not exist return error
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

	// If the path is not a directory return error
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

	// Read the directory
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

	// Iterate over the directory entries
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

		// Get the metadata of the entry
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

		// Create the file info
		let file = FileInfo {
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
	State(state): State<ServerState>,
	query_params: Query<DirectoryQuery>,
) -> (StatusCode, Json<SimpleServerResponse>) {
	let path = Path::new(&state.data_dir)
		.join(query_params.path.clone())
		.clean();

	// Confirm that the path is part of the data directory and is not the data
	// directory itsel
	if 
		!path.starts_with(&state.data_dir) ||
		path.ends_with(&state.data_dir)
	{
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
	State(state): State<ServerState>,
	query_params: Query<DirectoryQuery>,
) -> (StatusCode, Json<SimpleServerResponse>) {
	let path = Path::new(&state.data_dir)
		.join(query_params.path.clone())
		.clean();

	// Confirm that the path is part of the data directory and is not the data
	// directory itsel
	if 
		!path.starts_with(&state.data_dir) ||
		path.ends_with(&state.data_dir)
	{
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
	State(state): State<ServerState>,
	query_params: Query<MoveQuery>,
) -> (StatusCode, Json<SimpleServerResponse>) {
	let from = Path::new(&state.data_dir)
		.join(query_params.from.clone())
		.clean();

	let to = Path::new(&state.data_dir)
		.join(query_params.to.clone())
		.clean();

	// Confirm that the path is part of the data directory and is not the data
	// directory itsel
	if
		!from.starts_with(&state.data_dir) ||
		from.ends_with(&state.data_dir) ||
		!to.starts_with(&state.data_dir) ||
		to.ends_with(&state.data_dir)
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

/// Copy directory helper function
/// This function do not test if the source or destination directory exists
fn copy_dir_all(
	src: &PathBuf, dst: &PathBuf
) -> io::Result<()> {
	// Create the destination directory
	std::fs::create_dir_all(dst)?;

	let directories = src.read_dir()?;

	for entry in directories {
		let entry = entry?;
		let file_type = entry.file_type()?;
		let new_path = dst.join(entry.file_name());
		if file_type.is_dir() {
			copy_dir_all(&entry.path(), &new_path)?;
		} else {
			std::fs::copy(entry.path(), new_path)?;
		}
	}

	Ok(())
}

/// Return the file to download
async fn download_file(
	State(state): State<ServerState>,
	query_params: Query<DirectoryQuery>,
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
	State(state): State<ServerState>,
	query_params: Query<DirectoryQuery>,
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

/// Save a `Stream` to a file
async fn stream_to_file<S, E>(
	path: PathBuf, stream: S
) -> (StatusCode, Json<SimpleServerResponse>)
where
	S: Stream<Item = Result<Bytes, E>>,
	E: Into<BoxError>,
{
	let response = async {
		// Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream
			.map_err(
				|error| io::Error::new(io::ErrorKind::Other, error)
			);
        let body_reader =
			StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

		// Create the file
		let mut file = BufWriter::new(
			File::create(path).await?
		);

		// Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

		Ok::<(StatusCode, Json<SimpleServerResponse>), io::Error>((
			StatusCode::OK,
			Json(SimpleServerResponse {
				success: true,
				message: "File uploaded successfully".to_string(),
			})
		))
	}
	.await;

	match response {
		Ok(response) => response,
		Err(error) => (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(SimpleServerResponse {
				success: false,
				message: format!("Failed to upload file: {}", error),
			})
		),
	}
}

/// Upload a file
async fn upload_file(
	State(state): State<ServerState>,
	query_params: Query<DirectoryQuery>,
	request: Request
) -> (StatusCode, Json<SimpleServerResponse>) {
	// Join paths
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
				message: "File already exists".to_string(),
			})
		);
	}

	stream_to_file(path, request.into_body().into_data_stream()).await
}

// Copy the file or directory to the target directory
async fn copy(
	State(state): State<ServerState>,
	query_params: Query<MoveQuery>,
) -> (StatusCode, Json<SimpleServerResponse>) {
	let from = Path::new(&state.data_dir)
		.join(query_params.from.clone())
		.clean();

	let to = Path::new(&state.data_dir)
		.join(query_params.to.clone())
		.clean();

	// Confirm that the path is part of the data directory and is not the data
	// directory itsel
	if
		!from.starts_with(&state.data_dir) ||
		from.ends_with(&state.data_dir) ||
		!to.starts_with(&state.data_dir) ||
		to.ends_with(&state.data_dir)
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
	if !from.try_exists().unwrap_or(false)  {
		return (
			StatusCode::NOT_FOUND,
			Json(SimpleServerResponse {
				success: false,
				message: "Source file not found".to_string(),
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

	if from.is_dir() {
		// Copy the directory
		match copy_dir_all(&from, &to) {
			Ok(_) => return (
				StatusCode::OK,
				Json(SimpleServerResponse {
					success: true,
					message: "Directory copied successfully".to_string(),
				})
			),
			Err(err) => return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(SimpleServerResponse {
					success: false,
					message: format!("Failed to copy directory: {}", err),
				})
			),
		}
	} else if from.is_file() {
		// Copy the file
		match tokio::fs::copy(from, to).await {
			Ok(_) => return (
				StatusCode::OK,
				Json(SimpleServerResponse {
					success: true,
					message: "File copied successfully".to_string(),
				})
			),
			Err(err) => return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(SimpleServerResponse {
					success: false,
					message: format!("Failed to copy file: {}", err),
				})
			),
		}
	} else {
		return (
			StatusCode::BAD_REQUEST,
			Json(SimpleServerResponse {
				success: false,
				message: "Source is not a file or directory".to_string(),
			})
		);
	}
}
