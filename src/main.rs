use anyhow::{anyhow, Result};
use std::net::TcpStream;
use url::Url;
use std::io::{Read, Write, stdin, stdout};

// Helper Functions
fn read_text_content(stream: &mut TcpStream) -> Result<String> {
    let mut content = Vec::new();
    stream.read_to_end(&mut content)?;
    
    String::from_utf8(content).or_else(|err| {
        Ok(String::from_utf8_lossy(err.as_bytes()).into_owned())
    })
}

fn wait_for_enter() -> Result<()> {
    println!("Press Enter to continue...");
    stdin().read_line(&mut String::new())?;
    Ok(())
}

fn read_user_input() -> Result<String> {
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(input.trim().to_lowercase())
}

fn handle_error(error: &str) -> Result<()> {
    println!("\n{}", error);
    wait_for_enter()?;
    Ok(())
}

// UI Functions
fn display_separator() {
    println!("{}", "━".to_string().repeat(40));
}

fn display_header(url: &GopherUrl) {
    println!("\n=== Gopher Navigation ===");
    println!("Server: {}", url.host);
    println!("Port: {}", url.port);
    println!("Path: {}\n", url.selector);
    display_separator();
}

fn display_navigation_options(menu_items_len: usize, has_history: bool) {
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

fn display_loading_message(message: &str) {
    print!("\r{}", message);
    stdout().flush().unwrap();
}

fn display_content(content: &str) {
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

fn get_initial_url() -> Result<GopherUrl> {
    loop {
        println!("\n=== Gopher Client ===");
        println!("1. Connect to gopher://tramberend.de");
        println!("2. Connect to gopher://gopher.floodgap.com");
        println!("3. Enter custom Gopher URI");
        println!("q. Quit");
        print!("\nPlease choose: ");
        stdout().flush()?;

        match read_user_input()?.as_str() {
            "1" => return GopherUrl::parse("gopher://tramberend.de"),
            "2" => return GopherUrl::parse("gopher://gopher.floodgap.com"),
            "3" => {
                print!("Enter Gopher URI: ");
                stdout().flush()?;
                let uri = read_user_input()?;
                return GopherUrl::parse(&uri);
            }
            "q" | "Q" => std::process::exit(0),
            _ => println!("Invalid selection, please try again."),
        }
    }
}

fn handle_menu_selection(
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
            GopherItem::Info => {
                handle_error("Info items cannot be selected.")?;
            }
        }
    }
    Ok(true)
}

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
            },
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
            if line.trim().is_empty() { continue; }

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
                handle_menu_selection(selection, &menu_items, &mut current_url, &mut navigation_stack)?;
            }
        }
    }

    Ok(())
}

// Types and their implementations
#[derive(Debug, Clone)]
enum GopherItem {
    Text,
    Directory,
    Info,
}

#[derive(Debug, Clone)]
struct GopherUrl {
    host: String,
    port: u16,
    selector: String,
}

#[derive(Debug, Clone)]
struct MenuItem {
    item_type: GopherItem,
    display_text: String,
    selector: String,
    host: String,
    port: u16,
}

impl GopherUrl {
    fn parse(uri: &str) -> Result<Self> {
        let parsed = Url::parse(uri)?;
        if parsed.scheme() != "gopher" {
            return Err(anyhow!("Not a gopher URL: {}", uri));
        }

        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow!("No host found"))?
            .to_string();
        let port = parsed.port().unwrap_or(70);
        let selector = parsed.path().to_string();

        Ok(Self { host, port, selector })
    }

    fn connect(&self) -> Result<TcpStream> {
        let address = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(&address)
            .map_err(|e| anyhow!("Failed to connect to {}: {}", address, e))?;

        stream.write_all(format!("{}\r\n", self.selector).as_bytes())
            .map_err(|e| anyhow!("Failed to send selector: {}", e))?;

        Ok(stream)
    }
}

impl MenuItem {
    fn display(&self, index: Option<usize>) -> String {
        match self.item_type {
            GopherItem::Info => {
                format!("    {}", self.display_text)
            }
            _ => {
                let type_indicator = match self.item_type {
                    GopherItem::Directory => "[DIR]",
                    GopherItem::Text => "[TXT]",
                    GopherItem::Info => unreachable!(),
                };
                format!(
                    "[{}] {} {}",
                    index.expect("Non-info items should have an index"),
                    type_indicator,
                    self.display_text
                )
            }
        }
    }

    fn parse(line: &str) -> Result<Self> {
        if line.trim().is_empty() {
            return Err(anyhow!("Empty line"));
        }
    
        let item_type = match line.chars().next() {
            Some('0') => GopherItem::Text,
            Some('1') => GopherItem::Directory,
            Some('i') => GopherItem::Info,
            Some(c) => return Err(anyhow!("Unsupported item type: {}", c)),
            None => return Err(anyhow!("Empty line")),
        };
    
        let parts: Vec<&str> = line[1..].split('\t').collect();
        if parts.len() < 4 {
            return Err(anyhow!("Invalid menu item format"));
        }
    
        Ok(MenuItem {
            item_type,
            display_text: parts[0].trim().to_string(),
            selector: parts[1].to_string(),
            host: parts[2].to_string(),
            port: parts[3].parse()?,
        })
    }
}