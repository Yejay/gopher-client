use anyhow::Result;
use gopher_client::{
    models::{MenuItem, GopherItem},
    ui::{display_header, display_loading_message, display_navigation_options},
    handlers::handle_menu_selection,
    utils::{read_user_input, handle_error, get_initial_url},
};
use std::io::Read;

fn main() -> Result<()> {
    let mut navigation_stack = Vec::new();
    let mut current_url = get_initial_url()?;

    loop {
        display_header(&current_url);

        display_loading_message("Connecting to server...");
        let mut stream = match current_url.connect() {
            Ok(stream) => {
                println!("\rConnection established!");
                stream
            }
            Err(e) => {
                println!("\rConnection error: {}", e);
                println!("Press Enter to retry or 'q' to quit.");
                if read_user_input()?.eq("q") {
                    break;
                }
                continue;
            }
        };

        display_loading_message("Loading menu...");
        let mut response = String::new();
        if let Err(e) = stream.read_to_string(&mut response) {
            handle_error(&format!("Error reading menu: {}", e))?;
            continue;
        }
        println!("\rMenu loaded successfully!");

        let mut menu_items = Vec::new();
        let mut valid_index = 0;
        for line in response.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(item) = MenuItem::parse(line) {
                match item.item_type {
                    GopherItem::Info => println!("{}", item.display(None)),
                    _ => {
                        println!("{}", item.display(Some(valid_index)));
                        menu_items.push(item);
                        valid_index += 1;
                    }
                }
            }
        }

        display_navigation_options(menu_items.len(), !navigation_stack.is_empty());
        match read_user_input()?.as_str() {
            "q" => break,
            "b" => {
                if let Some(previous_url) = navigation_stack.pop() {
                    current_url = previous_url;
                } else {
                    handle_error("Already at root menu!")?;
                }
            }
            "r" => {
                println!("Reloading menu...");
                continue;
            }
            selection => {
                handle_menu_selection(
                    selection,
                    &menu_items,
                    &mut current_url,
                    &mut navigation_stack,
                )?;
            }
        }
    }

    Ok(())
}
