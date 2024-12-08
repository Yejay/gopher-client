pub mod models;
pub mod ui;
pub mod handlers;
pub mod utils;

// Re-export commonly used items for better public API
pub use models::{GopherItem, GopherUrl, MenuItem};
pub use handlers::{handle_menu_selection, handle_binary_file, handle_search};
pub use utils::{read_user_input, handle_error};
pub use ui::{display_header, display_loading_message, display_navigation_options};