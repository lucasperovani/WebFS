const BODY_ID = "#body";

const BREADCRUMB_LIST_ID = "#breadcrumb";
const LOADING_SPINNER_ID = "#loading_spinner";
const ERROR_DISPLAY_ID = "#error_display";

const FILE_CONTENT_ID = "#files_content";
const FILE_LIST_ID = "#files_list";

const ADD_FILE_BUTTON_ID = "#add_file";
const FILE_INPUT_ID = "#file_input";

const OPTIONS_MENU_ID = "#options_menu";
const ADD_FOLDER = "#add_folder";
const DELETE_FILE_ID = "#delete_file";

const FILE_DISPLAY_ID = "#file_display";
const FILE_DISPLAY_CLOSE_ID = "#close_file";
const FILE_NAME_DISPLAY_ID = "#file_name";
const DOWNLOAD_FILE_ID = "#download_file";

const PREV_FILE_ID = "#prev_file";
const NEXT_FILE_ID = "#next_file";

const TEXT_FILE_DISPLAY_ID = "#text_file_display";
const IMAGE_FILE_DISPLAY_ID = "#image_file_display";
const VIDEO_FILE_DISPLAY_ID = "#video_file_display";
const AUDIO_FILE_DISPLAY_ID = "#audio_file_display";
const PDF_FILE_DISPLAY_ID = "#pdf_file_display";
const UNKNOWN_FILE_DISPLAY_ID = "#unknown_file_display";

const DELETE_FILE_MODAL_ID = "#delete_file_warning_modal";
const DELETE_FILE_TEXT_ID = "#delete_file_warning_message";
const DELETE_FILE_MODAL_CANCEL_ID = "#cancel_delete";
const DELETE_FILE_MODAL_CONFIRM_ID = "#confirm_delete";

let long_press_timer;
let current_path = ".";
let file_list = [];

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

/**
 * Selects or deselects a file card.
 *
 * @param {HTMLElement} file_card - The file card to select.
 * @returns {void}
 */
function select_file_card(file_card) {
	// If delete warning is shown, return
	if ($(DELETE_FILE_MODAL_ID).is(":visible")) {
		return;
	}

	if ($(file_card).hasClass("selected")) {
		// If the card is already selected, deselect it
		$(file_card).removeClass("selected");
	} else {
		// Select the card
		$(file_card).addClass("selected");
	}
}

/**
 * Sets the height of the file name textarea.
 *
 * @param {HTMLElement} file_name_textarea - The file name textarea element.
 * @returns {void}
 */
function set_file_name_height(file_name_textarea) {
	file_name_textarea.style.height = 0;
	file_name_textarea.style.height = file_name_textarea.scrollHeight + "px";
}

/**
 * Allows the user to edit the file name of a file/folder.
 *
 * @param {HTMLElement} file_name_textarea - The file name textarea element.
 * @param {Event} event - The event that triggered the file name edit.
 * @returns {void}
 */
function file_name_edit(file_input, event) {
	// Stop the event to being propagated to parent elements
	event.stopPropagation();

	// Get the file name element
	$(file_input).attr("readonly", false);
}

/**
 * Renames a file or folder.
 *
 * @param {HTMLElement} file_input - The file input element.
 * @returns {void}
 */
function rename_file(file_input) {
	// Make the file input read-only again
	$(file_input).attr("readonly", true);

	// Get the new file name
	const new_file_name = $(file_input).val();

	// Get the old file name
	const old_file_name = $(file_input)
		.closest(".file-item")
		.data("file_name");

	// Skip if the file name is the same
	if (new_file_name === old_file_name) {
		return;
	}

	// Get the URL for the request
	const url  =
		"/api/v1/mv" +
		"?from=" + current_path + "/" + old_file_name +
		"&to=" + current_path + "/" + new_file_name;

	// Send the request to the server
	$.ajax({
		url: url,
		type: "PUT",
		success: function(data) {
			if (!data.success) {
				console.error("Error in rename_file success:", data.message);
				$(file_input).val(old_file_name);
				return;
			}

			// List the directory
			ls_directory(current_path);
		},
		error: function(error) {
			console.error("Error in rename_file error:", error);
			$(file_input).val(old_file_name);
		}
	});
}

function get_file(file_name) {
	// Get the URL for the request
	const path = "path=" + current_path + "/" + file_name;
	let url  = "/api/v1/download?" + path;

	// Stop any media playing
	$(AUDIO_FILE_DISPLAY_ID).trigger("pause");
	$(VIDEO_FILE_DISPLAY_ID).trigger("pause");

	// Hide everything
	$(TEXT_FILE_DISPLAY_ID).hide();
	$(IMAGE_FILE_DISPLAY_ID).hide();
	$(VIDEO_FILE_DISPLAY_ID).hide();
	$(AUDIO_FILE_DISPLAY_ID).hide();
	$(PDF_FILE_DISPLAY_ID).hide();
	$(UNKNOWN_FILE_DISPLAY_ID).hide();

	// File name
	$(FILE_NAME_DISPLAY_ID).text(file_name);

	// Get the file from the file list
	const fileIndex = file_list.findIndex((file) => file.name === file_name);

	// If file not found, return
	if (fileIndex < 0 || !file_list.length) {
		console.error("File not found in file list:", file_name);
		return;
	}

	// Set the Download button URL
	$(DOWNLOAD_FILE_ID).attr("href", url);

	// Add the peek query parameter to only get the source
	url += "&peek=true";
	
	// Get the file and the previous and next file
	const file = file_list[fileIndex];
	const previousFile =
		file_list[fileIndex - 1] || file_list[file_list.length - 1];
	const nextFile =
		file_list[fileIndex + 1] || file_list[0];

	// Set the previous and next file buttons
	$(PREV_FILE_ID).attr("onclick", "get_file('" + previousFile.name + "')");
	$(NEXT_FILE_ID).attr("onclick", "get_file('" + nextFile.name + "')");
	
	// Check file type
	if (file.mime.includes("image/")) {
		// If image just set the source and show
		$(IMAGE_FILE_DISPLAY_ID)
			.attr("src", url)
			.show();
	} else if (file.mime.includes("text/")) {
		// If text file, get the text and show
		$.ajax({
			url: url,
			type: "GET",
			success: function(data) {
				$(TEXT_FILE_DISPLAY_ID)
					.text(data)
					.show();
			},
			error: function(error) {
				console.error("Error in get_file error:", error);
			}
		});
	} else if (file.mime.includes("video/")) {
		// If video file, set the source and show
		$(VIDEO_FILE_DISPLAY_ID)
			.attr("src", url)
			.show();
	} else if (file.mime.includes("audio/")) {
		// If audio file, set the source and show
		$(AUDIO_FILE_DISPLAY_ID)
			.attr("src", url)
			.show();
	} else if (file.mime.includes("application/pdf")) {
		// If PDF file, set the source and show
		$(PDF_FILE_DISPLAY_ID)
			.attr("src", url)
			.show();
	} else {
		// If unknown file type, just show the download button
		$(UNKNOWN_FILE_DISPLAY_ID).show();
	}

	// Show the file display
	$(FILE_DISPLAY_ID).show();
}

/**
 * Returns the HTML for a file or directory card.
 *
 * @param {object} file - The file object.
 * @param {boolean} is_dir - Whether the file is a directory.
 * @returns {string} The HTML for the file card.
 */
function file_card_html(file, is_dir) {
	// File name string
	const filename = $("<textarea>")
		.addClass("card-text text-center bg-transparent w-100 border-0")
		.css({
			"resize": "none",
			"box-sizing": "border-box",
			"height": "28px",
			"overflow-y": "hidden"
		})
		.attr("readonly", "true")
		.attr("onclick", "event.stopPropagation()")
		.attr("ondblclick", "file_name_edit(this, event)")
		.attr("onblur", "rename_file(this)")
		.attr("oninput", "set_file_name_height(this)")
		.attr("maxlength", "255")
		.append(file.name);

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
		.attr("onclick", "select_file_card(this)")
		.attr("data-file_name", file.name)
		.attr("data-is_dir", is_dir)
		.append(icon)
		.append(card_body);

	// Call function
	const call_function = is_dir ?
		"ls_directory(\"" + current_path + "/" + file.name + "\")" :
		"get_file(\"" + file.name + "\")";

	return $("<div>")
		.addClass("col-6 col-md-3 col-lg-2")
		.append(card)
		.attr("ondblclick", call_function)
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

	// Clean the file list
	file_list = [];

	// Loop through the files
	const files_html = data.files.map((file) => {
		// Add only files, not directories, to the file list
		if (!file.is_dir) {
			file_list.push(file);
		}

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
 * Upload single file to the current directory, sending it to the server.
 *
 * @param {string} file_name - The file name to upload.
 * @param {ArrayBuffer} file_data - The file data to upload.
 *
 * @returns {void}
 */
function upload_single_file(file_name, file_data) {
	// Send the request to the server
	$.ajax({
		url: "/api/v1/upload?path=" + current_path + "/" + file_name,
		type: "PUT",
		data: file_data,
		processData: false,
		contentType: false,
		success: function(data) {
			if (!data.success) {
				show_error();
				return;
			}

			// List the directory
			ls_directory(current_path);
		},
		error: function(error) {
			console.error("Error in upload_files:", error);
			show_error();
		}
	});
}

/**
 * Uploads many files to the current directory, sending it to the server.
 *
 * @param {Event} event - The change event received.
 *
 * @returns {void}
 */
async function upload_files(event) {
	// Spin front-end
	show_spinner();

	// Get the file, file name, and it's buffer
	const files = event.target.files;

	// Skip if no files
	if (!files.length) return;

	// Loop through the files
	for (const file of files) {
		const file_name = file.name;
		const file_buffer = await file.arrayBuffer();

		upload_single_file(file_name, file_buffer);
	}
}

/**
 * Shows the options menu.
 *
 * @param {Event} event - The event that triggered the options menu.
 * @returns {void}
 */
function show_options_menu(event) {
	// Only show the options menu if delete warning is not shown
	if ($(DELETE_FILE_MODAL_ID).is(":visible")) {
		return;
	}

	// Set the position of the options menu
	$(OPTIONS_MENU_ID).css({
		top: event.clientY - 15,
		left: event.clientX - 15,
	});

	// Show the options menu
	$(OPTIONS_MENU_ID).show();
	$(OPTIONS_MENU_ID).find("ul").addClass("d-block");

	// Prevent the default context menu
	event.preventDefault();
}

/**
 * Hides the options menu.
 *
 * @returns {void}
 */
function hide_options_menu() {
	$(OPTIONS_MENU_ID).hide();
	$(OPTIONS_MENU_ID).find("ul").removeClass("d-block");
}

/**
 * Deletes the selected files.
 *
 * @returns {void}
 */
function delete_files() {
	// Hide the delete warning
	$(DELETE_FILE_MODAL_ID).hide();

	// Get the selected files
	const selected_files = $(".file-item.selected");

	// Skip if no files are selected
	if (!selected_files.length) return;

	// Loop through the selected files
	for (const file of selected_files) {
		// Get the file name and file type
		const file_name = $(file).data("file_name");
		const is_folder = $(file).data("is_dir");

		// Build the URL for the request based on file/folder
		const delete_type = is_folder ? "rmdir" : "rm";
		const url  =
			"/api/v1/" + delete_type +
			"?path=" + current_path + "/" + file_name;

		// Send the request to the server
		$.ajax({
			url: url,
			type: "DELETE",
			success: function(data) {
				if (!data.success) {
					show_error();
					return;
				}

				// List the directory
				ls_directory(current_path);

				// Hide the options menu
				hide_options_menu();
			},
			error: function(error) {
				console.error("Error in delete_files:", error);
				show_error();
			}
		});
	}
}

/**
 * Shows the delete warning.
 *
 * @returns {void}
 */
function show_delete_warning() {
	// Hide the options menu
	hide_options_menu();

	// Get the selected files
	const selected_files = $(".file-item.selected");

	// Skip if no files are selected
	if (!selected_files.length) return;

	// Display the message based on the number of files
	$(DELETE_FILE_TEXT_ID).text(
		`Tem certeza que deseja excluir ${selected_files.length} arquivo(s)?`,
	);

	// Show the delete warning
	$(DELETE_FILE_MODAL_ID).show();
}

/**
 * Creates a new folder in the current directory.
 *
 * @param {HTMLElement} folder_name_textarea - The folder name textarea element.
 * @returns {void}
 */
function create_folder(folder_name_textarea) {

	// Get the new folder name
	const folder_name = $(folder_name_textarea).val();

	// Get the URL for the request
	const url  = "/api/v1/mkdir?path=" + current_path + "/" + folder_name;

	// Send the request to the server
	$.ajax({
		url: url,
		type: "PUT",
		success: function(data) {
			if (!data.success) {
				console.error("Error in create_folder success:", data.message);
			}

			// List the directory
			ls_directory(current_path);
		},
		error: function(error) {
			console.error("Error in create_folder error:", error);
			ls_directory(current_path);
		}
	});
}

/**
 * Adds a folder to the current directory.
 *
 * @returns {void}
 */
function add_folder(event) {
	// Stop link click
	event.preventDefault();

	// Hide the options menu
	hide_options_menu();

	// File name string
	const filename = $("<textarea>")
		.addClass("card-text text-center bg-transparent w-100 border-0")
		.css({
			"resize": "none",
			"box-sizing": "border-box",
			"height": "28px",
			"overflow-y": "hidden"
		})
		.attr("onblur", "create_folder(this)")
		.attr("oninput", "set_file_name_height(this)")
		.attr("maxlength", "255")
		.append("Nova Pasta");

	// Card body with the file name
	const card_body = $("<div>")
		.addClass("card-body")
		.append(filename);

	// File icon element
	const icon = $("<i>")
		.addClass("fas card-img-top text-center file-item-icon mt-3")
		.addClass("fa-folder folder");

	// Card element with the icon and body
	const card = $("<div>")
		.addClass("card mb-4 box-shadow file-item")
		.append(icon)
		.append(card_body);

	// Add the folder to the page
	const folder =  $("<div>")
		.addClass("col-md-3")
		.append(card);

	$(FILE_LIST_ID)
		.append(folder)
		.find('textarea:last')
		.focus()
		.select();
}

/**
 * When HTML document is ready, add event listeners.
 */
$(document).ready(function() {
	// Get the home directory
	ls_directory(".");

	// Add upload file button action
	$(ADD_FILE_BUTTON_ID).on("click", () => $(FILE_INPUT_ID)[0].click());

	// Add file input change action
	$(FILE_INPUT_ID).on("change", upload_files);

	// Add click event to use custom options menu
	document.addEventListener('contextmenu', show_options_menu, false);

	// Add event to hide options menu when mouse leaves
	$(OPTIONS_MENU_ID).on("mouseleave", hide_options_menu);

	// Add long press event to use custom options menu
	$(BODY_ID).on("pointerdown", function(event) {
		long_press_timer = setTimeout(() => show_options_menu(event), 250);
	}).on("pointerup", function() {
		clearTimeout(long_press_timer);
	});

	// Add create folder action
	$(ADD_FOLDER).on("click", add_folder);

	// Add delete file action
	$(DELETE_FILE_ID).on("click", show_delete_warning);

	// Add delete file modal cancel action
	$(DELETE_FILE_MODAL_CANCEL_ID).on(
		"click", () => $(DELETE_FILE_MODAL_ID).hide()
	);

	// Add delete file modal confirm action
	$(DELETE_FILE_MODAL_CONFIRM_ID).on("click", delete_files);

	// On File Display Close
	$(FILE_DISPLAY_CLOSE_ID).on("click", () => {
		// Stop any media playing
		$(AUDIO_FILE_DISPLAY_ID).trigger("pause");
		$(VIDEO_FILE_DISPLAY_ID).trigger("pause");

		// Hide the file display
		$(FILE_DISPLAY_ID).hide();
	});
});
