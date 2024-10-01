use std::{io, path::{Path, PathBuf}};

use axum::{extract::{Query, State}, http::StatusCode, Json};
use path_clean::PathClean;

use super::types::{MoveQuery, ServerState, SimpleServerResponse};

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

/// Copy the file or directory to the target directory
///
/// This route copies a file or directory to the target directory.
pub async fn copy(
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
