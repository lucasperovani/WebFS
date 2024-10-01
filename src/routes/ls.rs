use std::path::Path;

use axum::{extract::{Query, State}, http::StatusCode, Json};
use path_clean::PathClean;

use super::types::{DirectoryQuery, FileInfo, LsDirectoryResponse, ServerState};

/// List the files in a directory
///
/// This route lists the files in a directory. The path to the directory is
/// provided as a query parameter. The path is relative to the data directory.
/// The route returns a JSON response with the list of files in the directory.
/// The response includes the name, size, and mime type of each file.
/// The route returns an error if the path is invalid, the directory does not
/// exist, or the path is not a directory.
///
/// # Example
/// ```http
/// GET /ls?path=example
/// ```
/// 
/// # Query Parameters
/// - `path`: The path to the directory to list. The path is relative to the
///  data directory.
///
/// # Response
/// The response is a JSON object with the following fields:
/// - `success`: A boolean indicating if the request was successful.
/// - `message`: A message describing the result of the request.
/// - `files`: An array of file objects. Each file object has the following
///  fields:
///  - `name`: The name of the file.
/// - `size`: The size of the file in bytes.
/// - `is_dir`: A boolean indicating if the file is a directory.
/// - `mime`: The mime type of the file. This field is `null` for directories.
pub async fn ls_dir(
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

		// Get the mime type of the file
		let mime = if metadata.is_dir() {
			None
		} else {
			Some(
				mime_guess::from_path(&entry.path())
					.first_or_octet_stream()
					.to_string()
			)
		};

		// Create the file info
		let file = FileInfo {
			name: entry
				.file_name()
				.into_string()
				.unwrap_or("unknown".to_string()),
			size: metadata.len(),
			is_dir: metadata.is_dir(),
			mime: mime,
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