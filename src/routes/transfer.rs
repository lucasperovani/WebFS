use std::{io, path::{Path, PathBuf}};

use path_clean::PathClean;
use tokio::{io::BufWriter, fs::File};
use tokio_util::io::{ReaderStream, StreamReader};
use axum::{
	body::{Body, Bytes},
	extract::{Query, Request, State},
	http::{header, StatusCode},
	response::IntoResponse,
	BoxError,
    Json,
};
use futures::{Stream, TryStreamExt};

use super::types::{
    DirectoryQuery,
    DownloadDirectoryQuery,
    ServerState,
    SimpleServerResponse,
};

/// Return the file to download
///
/// This route returns the file to download.
pub async fn download_file(
	State(state): State<ServerState>,
	query_params: Query<DownloadDirectoryQuery>,
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

	// Guess the mime type of the file
	let mime = mime_guess::from_path(&path)
		.first_or_octet_stream();

	// Format Dispotion header
	let disposition = format!(
		"attachment; filename={:?}",
		path.file_name().unwrap_or(std::ffi::OsStr::new("unknown"))
	);

	// Set the headers
	let headers = match query_params.peek.unwrap_or(false) {
		false => [
			(header::CONTENT_TYPE, format!("{}", mime)),
			(header::CONTENT_DISPOSITION, disposition)
		],
		true => [
			(header::CONTENT_TYPE, format!("{}", mime)),
			(header::CONTENT_DISPOSITION, "".to_string())
		],
	};
	
	Ok((headers, body).into_response())
}


/// Save a `Stream` to a file
///
/// This function takes a `Stream` and saves it to a file. It returns a tuple
/// containing the status code and a `Json` response.
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
			File::create(&path).await?
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
		Err(error) => {
			// Try deleting the file if an error occurs
			let _ = tokio::fs::remove_file(path).await;

			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(SimpleServerResponse {
					success: false,
					message: format!("Failed to upload file: {}", error),
				})
			);
		}
	}
}

/// Upload a file
///
/// This route uploads a file to the server.
pub async fn upload_file(
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
