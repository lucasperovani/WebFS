
use serde::{Deserialize, Serialize};

/// The state of the server
/// 
/// This struct holds the state of the server. The state includes the path to
/// the data directory. The state is shared between the routes and the server.
#[derive(Clone)]
pub struct ServerState {
	pub data_dir: String,
}

/// The response to the ls directory request
///
/// This struct is used to serialize the response to the ls directory request.
/// The response includes a boolean indicating if the request was successful,
/// a message describing the result of the request, and an array of file
/// objects.
#[derive(Serialize)]
pub struct FileInfo {
	pub name: String,
	pub size: u64,
	pub is_dir: bool,
	pub mime: Option<String>,
}

/// The response to the ls directory request
///
/// This struct is used to serialize the response to the ls directory request.
#[derive(Serialize)]
pub struct LsDirectoryResponse {
	pub success: bool,
	pub message: String,
	pub files: Vec<FileInfo>
}

/// The response to the download directory request
///
/// This struct is used to serialize the request to the download directory.
#[derive(Deserialize)]
pub struct DirectoryQuery {
	pub path: String,
}

/// The response to the download directory request
///
/// This struct is used to serialize the response to the download directory.
#[derive(Deserialize)]
pub struct DownloadDirectoryQuery {
	pub path: String,
	pub peek: Option<bool>,
}

/// The response to the move directory request
///
/// This struct is used to serialize the response to the move directory.
#[derive(Deserialize)]
pub struct MoveQuery {
	pub from: String,
	pub to: String,
}

/// The response to the simple server request
///
/// This struct is used to serialize the response to the simple server request.
#[derive(Serialize)]
pub struct SimpleServerResponse {
	pub success: bool,
	pub message: String,
}
