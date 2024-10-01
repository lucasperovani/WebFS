use std::path::Path;

use axum::{extract::{Query, State}, http::StatusCode, Json};
use path_clean::PathClean;

use super::types::{DirectoryQuery, ServerState, SimpleServerResponse};

/// Create directory
///
/// This route creates a directory in the data directory.
///
/// # Example
/// ```http
/// POST /mk?path=example
/// ```
pub async fn create_dir(
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