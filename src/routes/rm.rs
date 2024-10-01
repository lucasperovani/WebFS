use std::path::Path;

use axum::{extract::{Query, State}, http::StatusCode, Json};
use path_clean::PathClean;

use super::types::{DirectoryQuery, ServerState, SimpleServerResponse};

/// Delete whole directory
///
/// This route deletes a directory in the data directory.
pub async fn delete_dir(
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

/// Remove a file
///
/// This route deletes a file in the data directory.
pub async fn delete_file(
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
