use crate::models::GopherUrl;
use std::io::{stdout, Write};

pub fn display_separator() {
    println!("{}", "━".to_string().repeat(40));
}

pub fn display_header(url: &GopherUrl) {
    println!("\n=== Gopher Navigation ===");
    println!("Server: {}", url.host);
    println!("Port: {}", url.port);
    println!("Path: {}\n", url.selector);
    display_separator();
}

pub fn display_navigation_options(menu_items_len: usize, has_history: bool) {
    display_separator();
    println!("Navigation Options:");
    println!("  (Q)uit   - Exit program");
    if has_history {
        println!("  (B)ack   - Return to previous menu");
    }
    println!("  (R)eload - Refresh current menu");
    if menu_items_len > 0 {
        println!("  [0-{}]   - Select menu item", menu_items_len - 1);
    }
    print!("\nChoice: ");
    stdout().flush().unwrap();
}

pub fn display_loading_message(message: &str) {
    print!("\r{}", message);
    stdout().flush().unwrap();
}

pub fn display_content(content: &str) {
    display_separator();
    println!("Content:");
    display_separator();
    
    for line in content.lines() {
        println!("{}", line);
    }
    
    display_separator();
    println!("\nPress Enter to return to menu...");
    stdout().flush().unwrap();
}