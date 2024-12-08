use crate::models::{GopherItem, GopherUrl, MenuItem};
use crate::ui::*;
use crate::utils::*;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;

pub fn handle_menu_selection(
    selection: &str,
    menu_items: &[MenuItem],
    current_url: &mut GopherUrl,
    navigation_stack: &mut Vec<GopherUrl>,
) -> Result<bool> {
    if let Ok(num) = selection.parse::<usize>() {
        if num >= menu_items.len() {
            handle_error("Invalid selection!")?;
            return Ok(true);
        }

        let selected = &menu_items[num];
        match selected.item_type {
            GopherItem::Directory => {
                navigation_stack.push(current_url.clone());
                *current_url = GopherUrl {
                    host: selected.host.clone(),
                    port: selected.port,
                    selector: selected.selector.clone(),
                };
            }
            GopherItem::Text => {
                display_loading_message("Fetching content...");
                let mut stream = TcpStream::connect(format!(
                    "{}:{}",
                    selected.host, selected.port
                ))?;
                stream.write_all(format!("{}\r\n", selected.selector).as_bytes())?;
                let content = read_text_content(&mut stream)?;
                display_content(&content);
                wait_for_enter()?;
            }
            GopherItem::Search => {
                navigation_stack.push(current_url.clone());
                let search_selector = handle_search(&GopherUrl {
                    host: selected.host.clone(),
                    port: selected.port,
                    selector: selected.selector.clone(),
                })?;
                *current_url = GopherUrl {
                    host: selected.host.clone(),
                    port: selected.port,
                    selector: search_selector,
                };
            }
            GopherItem::Image => {
                handle_binary_file(selected)?;
            }
            GopherItem::Info => {
                handle_error("Info items cannot be selected.")?;
            }
        }
    }
    Ok(true)
}

pub fn handle_binary_file(item: &MenuItem) -> Result<()> {
    println!("\nBinary file options:");
    println!("1. Download file");
    println!("2. Download and open with default program");
    println!("3. Cancel");
    print!("\nChoice: ");
    std::io::stdout().flush()?;

    match read_user_input()?.as_str() {
        "1" => {
            download_file(item)?;
            Ok(())
        }
        "2" => {
            let file_path = download_file(item)?;
            open_file(&file_path)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn download_file(item: &MenuItem) -> Result<PathBuf> {
    display_loading_message("Connecting to server...");
    let mut stream = TcpStream::connect(format!("{}:{}", item.host, item.port))?;

    stream.write_all(format!("{}\r\n", item.selector).as_bytes())?;

    // Create downloads directory if it doesn't exist
    std::fs::create_dir_all("downloads")?;

    // Generate filename from the selector or display text
    let filename = if let Some(name) = item.selector.split('/').last() {
        if !name.is_empty() {
            name
        } else {
            &item.display_text
        }
    } else {
        &item.display_text
    };

    let file_path = PathBuf::from("downloads").join(filename);

    display_loading_message("Downloading file...");
    let mut file = File::create(&file_path)?;
    std::io::copy(&mut stream, &mut file)?;

    println!("\rFile downloaded to: {}", file_path.display());
    Ok(file_path)
}

fn open_file(path: &PathBuf) -> Result<()> {
    #[cfg(target_os = "windows")]
    let command = "start";
    #[cfg(target_os = "macos")]
    let command = "open";
    #[cfg(target_os = "linux")]
    let command = "xdg-open";

    Command::new(command)
        .arg(path)
        .spawn()
        .map_err(|e| anyhow!("Failed to open file: {}", e))?;

    Ok(())
}

pub fn handle_search(url: &GopherUrl) -> Result<String> {
    print!("Enter search query: ");
    std::io::stdout().flush()?;
    let query = read_user_input()?;
    Ok(format!("{}\t{}", url.selector, query))
}