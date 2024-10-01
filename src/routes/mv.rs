use std::path::Path;

use axum::{extract::{Query, State}, http::StatusCode, Json};
use path_clean::PathClean;

use super::types::{MoveQuery, ServerState, SimpleServerResponse};

/// Move directory
///
/// This route moves a directory in the data directory.
pub async fn move_dir_or_file(
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
