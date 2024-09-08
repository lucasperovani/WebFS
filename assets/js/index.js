const BREADCRUMB_LIST_ID = "#breadcrumb";
const LOADING_SPINNER_ID = "#loading_spinner";
const ERROR_DISPLAY_ID = "#error_display";
const FILE_CONTENT_ID = "#files_content";
const FILE_LIST_ID = "#files_list";
const ADD_FILE_BUTTON_ID = "#add_file";

let current_path = ".";

/**
 * Returns the breadcrumb element for the home page.
 *
 * @returns {string} The breadcrumb element.
 */
function get_home_breadcrumb() {
	// Create the link element
	const link = $("<a>")
		.addClass("link-body-emphasis fw-semibold text-decoration-none")
		.attr("href", "#")
		.append(
			$("<i>").addClass("fa-solid fa-house"),
			" Home "
		);

	// Create a list item element
	return $("<li>")
		.addClass("breadcrumb-item")
		.attr("onclick", "ls_directory('.')")
		.append(link)
		.prop('outerHTML');
}

/**
 * Converts a path to a breadcrumb HTML element as string. 
 *
 * @param {string} path - The path to convert to a breadcrumb.
 * @returns {string} The breadcrumb HTML element as string.
 */
function convert_path_to_breadcrumb(path) {
	// Split the path into parts
	const parts = path.split("/");

	// Create home breadcrumb item
	const home_breadcrumb = get_home_breadcrumb();

	// Create the breadcrumb element
	const breadcrumb = parts.map((part, index) => {
		// Skip . and ..
		if (part === "." || part === "..") {
			return "";
		}

		// Create the link element
		const link = $("<a>")
			.addClass("link-body-emphasis fw-semibold text-decoration-none")
			.attr("href", "#")
			.append(part);

		// Create the call function
		const call_function = "ls_directory('" +
			parts.slice(0, index + 1).join("/") +
		"')";

		// Create a list item element
		return $("<li>")
			.addClass("breadcrumb-item")
			.attr("onclick", call_function)
			.append(link)
			.prop('outerHTML');
	}).join("");

	// Return the breadcrumb element
	return home_breadcrumb + breadcrumb;
}

function file_card_html(file, is_dir) {
	// File name string
	const filename = $("<p>")
		.addClass("card-text text-center")
		.text(file.name);

	// Card body with the file name
	const card_body = $("<div>")
		.addClass("card-body")
		.append(filename);

	// File icon element
	const icon = $("<i>")
		.addClass("fas card-img-top text-center file-item-icon mt-3")
		.addClass(is_dir ? "fa-folder folder" : "fa-file file");

	// Card element with the icon and body
	const card = $("<div>")
		.addClass("card mb-4 box-shadow file-item")
		.append(icon)
		.append(card_body);

	// Call function
	const call_function = is_dir ?
		"ls_directory(\"" + current_path + "/" + file.name + "\")" : "";

	return $("<div>")
		.addClass("col-md-3")
		.append(card)
		.attr("onclick", call_function)
		.prop('outerHTML');
}

/**
 * Loads the file list into the page.
 *
 * @param {object} data - The data to load.
 * @returns {void}
 */
function load_file_list(data) {
	// Skip on error
	if (!data.success) {
		console.error(data.message);
		return;
	}

	// Loop through the files
	const files_html = data.files.map((file) => {
		return file_card_html(file, file.is_dir);
	}).join("");

	// Add the files to the page
	$(FILE_LIST_ID).html(files_html);
}

/**
 * Shows the files.
 *
 * @returns {void}
 */
function show_file_list() {
	$(FILE_CONTENT_ID).show();
	$(LOADING_SPINNER_ID).hide();
	$(ERROR_DISPLAY_ID).hide();
}

/**
 * Shows the spinner.
 *
 * @returns {void}
 */
function show_spinner() {
	$(LOADING_SPINNER_ID).show();
	$(ERROR_DISPLAY_ID).hide();
	$(FILE_CONTENT_ID).hide();
}

/**
 * Shows the error display.
 *
 * @returns {void}
 */
function show_error() {
	$(ERROR_DISPLAY_ID).show();
	$(LOADING_SPINNER_ID).hide();
	$(FILE_CONTENT_ID).hide();
}

/**
 * Lists the contents of a directory.
 *
 * @param {string} path - The path to list.
 * @returns {void}
 */
function ls_directory(path) {
	// Spin front-end
	show_spinner();

	// Update the current path
	current_path = path;

	// Send a request to the server
	$.ajax({
		url: "/api/v1/ls?path=" + path,
		type: "GET",
		success: function(data) {
			if (!data.success) {
				show_error();
				return;
			}

			// Convert the path to a breadcrumb
			const breadcrumb = convert_path_to_breadcrumb(path);

			// Add the breadcrumb element to the page
			$(BREADCRUMB_LIST_ID)
				.empty()
				.append(breadcrumb);

			// Load the file list
			load_file_list(data);
			show_file_list();
		},
		error: function(error) {
			console.error("Error in ls dir:", error);
			show_error();
		}
	});
}

/**
 * Adds a file to the current directory.
 *
 * @returns {void}
 */
function add_file() {
}

$(document).ready(function() {
	// Get the home directory
	ls_directory(".");
});