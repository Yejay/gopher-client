use anyhow::{anyhow, Result};
use std::net::TcpStream;
use url::Url;
use std::io::{Read, Write, stdin, stdout};


fn read_text_content(stream: &mut TcpStream) -> Result<String> {
    let mut content = Vec::new();
    stream.read_to_end(&mut content)?;
    
    String::from_utf8(content).or_else(|err| {
        // If UTF-8 conversion fails, use the bytes for lossy conversion
        Ok(String::from_utf8_lossy(err.as_bytes()).into_owned())
    })
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

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => return GopherUrl::parse("gopher://tramberend.de"),
            "2" => return GopherUrl::parse("gopher://gopher.floodgap.com"),
            "3" => {
                print!("Enter Gopher URI: ");
                stdout().flush()?;
                let mut uri = String::new();
                stdin().read_line(&mut uri)?;
                return GopherUrl::parse(uri.trim());
            }
            "q" | "Q" => std::process::exit(0),
            _ => {
                println!("Invalid selection, please try again.");
                continue;
            }
        }
    }
}


fn main() -> Result<()> {
    // Initialize navigation history
    let mut navigation_stack = Vec::new();
    let mut current_url = get_initial_url()?;

    loop {
        println!("\n=== Gopher Menu Navigation ===");
        println!("Current server: {}", current_url.host);
        
        // Connect and get menu content
        let mut stream = match current_url.connect() {
            Ok(stream) => stream,
            Err(e) => {
                println!("Connection error: {}. Press Enter to retry or 'q' to quit.", e);
                let mut input = String::new();
                stdin().read_line(&mut input)?;
                if input.trim().eq_ignore_ascii_case("q") {
                    break;
                }
                continue;
            }
        };

        let mut response = String::new();
        if let Err(e) = stream.read_to_string(&mut response) {
            println!("Error reading response: {}", e);
            continue;
        }

        // Parse and display menu items
        let mut menu_items = Vec::new();
        let mut valid_index = 0;

        for line in response.lines() {
            if line.trim().is_empty() { continue; }

            match MenuItem::parse(line) {
                Ok(item) => match item.item_type {
                    GopherItem::Info => {
                        println!("{}", item.display(None));
                    }
                    _ => {
                        println!("{}", item.display(Some(valid_index)));
                        menu_items.push(item);
                        valid_index += 1;
                    }
                },
                Err(_) => continue,
            }
        }

        println!("\nOptions:");
        println!("(Q)uit   - Exit the program");
        println!("(B)ack   - Go to previous menu");
        println!("(R)eload - Reload current menu");
        print!("\nPlease choose (Q/B/R or 0-{}): ", menu_items.len() - 1);
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "q" => break,
            "b" => {
                if let Some(previous_url) = navigation_stack.pop() {
                    current_url = previous_url;
                } else {
                    println!("Cannot go back - at root menu. Press Enter to continue...");
                    stdin().read_line(&mut String::new())?;
                }
            }
            "r" => continue, // Will reload on next loop iteration
            selection => {
                if let Ok(num) = selection.parse::<usize>() {
                    if num < menu_items.len() {
                        let selected = &menu_items[num];
                        match selected.item_type {
                            GopherItem::Directory => {
                                navigation_stack.push(current_url.clone());
                                current_url = GopherUrl {
                                    host: selected.host.clone(),
                                    port: selected.port,
                                    selector: selected.selector.clone(),
                                };
                            }
                            GopherItem::Text => {
                                println!("\nFetching text content...");
                                let mut stream = TcpStream::connect(format!(
                                    "{}:{}",
                                    selected.host, selected.port
                                ))?;
                                stream.write_all(format!("{}\r\n", selected.selector).as_bytes())?;
                                let content = read_text_content(&mut stream)?;
                                println!("\n{}", content);
                                println!("\nPress Enter to continue...");
                                stdin().read_line(&mut String::new())?;
                            }
                            GopherItem::Info => {
                                println!("Info item - no action needed");
                            }
                        }
                    } else {
                        println!("Invalid selection! Press Enter to continue...");
                        stdin().read_line(&mut String::new())?;
                    }
                }
            }
        }
    }

    Ok(())
}


#[derive(Debug, Clone)]
enum GopherItem {
    Text,
    Directory,
    Info,
}

#[derive(Debug, Clone)]
struct GopherUrl {
    host: String,
    // unsigned (non-negative) 16-bit integer (0 to 65535)
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

        Ok(Self {
            host,
            port,
            selector,
        })
    }

    fn connect(&self) -> Result<TcpStream> {
        let address = format!("{}:{}", self.host, self.port);
        println!("Attempting to connect to: {}", address); // Debug print

        let mut stream = TcpStream::connect(&address)?;
        println!("Connection established!"); // Debug print

        let selector_string = format!("{}\r\n", self.selector);
        println!("Sending selector: {:?}", selector_string); // Debug print

        stream.write_all(selector_string.as_bytes())?;
        println!("Selector sent successfully!"); // Debug print

        Ok(stream)
    }
}

impl MenuItem {
    fn display(&self, index: Option<usize>) -> String {
        match self.item_type {
            GopherItem::Info => {
                // Info items don't get an index
                format!("    {}", self.display_text)
            }
            _ => {
                let type_indicator = match self.item_type {
                    GopherItem::Directory => "[DIR]",
                    GopherItem::Text => "[TXT]",
                    GopherItem::Info => unreachable!(), // Already handled
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
        // Skip empty lines
        if line.trim().is_empty() {
            return Err(anyhow!("Empty line"));
        }
    
        // Get the item type from first character
        let item_type = match line.chars().next() {
            Some('0') => GopherItem::Text,
            Some('1') => GopherItem::Directory,
            Some('i') => GopherItem::Info,
            Some(c) => return Err(anyhow!("Unsupported item type: {}", c)),
            None => return Err(anyhow!("Empty line")),
        };
    
        // Split the remaining line by tabs
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
