use anyhow::{anyhow, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use url::Url;


fn read_text_content(stream: &mut TcpStream) -> Result<String> {
    let mut content = Vec::new();
    stream.read_to_end(&mut content)?;
    
    String::from_utf8(content).or_else(|err| {
        // If UTF-8 conversion fails, use the bytes for lossy conversion
        Ok(String::from_utf8_lossy(err.as_bytes()).into_owned())
    })
}
fn main() -> Result<()> {
    let mut current_url = GopherUrl::parse("gopher://gopher.floodgap.com:70")?;

    loop {
        // Connect and get menu content
        let mut stream = current_url.connect()?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        // Parse and display menu items
        let mut menu_items = Vec::new();
        let mut valid_index = 0; // Track actual menu index separately

        for line in response.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match MenuItem::parse(line) {
                Ok(item) => match item.item_type {
                    GopherItem::Info => {
                        println!("{}", item.display(None));
                        continue;
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

        println!("\nCurrent server: {}", current_url.host);
        println!("Enter a number to select, 'q' to quit:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "q" => {
                println!("Goodbye!");
                break;
            }
            selection => {
                if let Ok(num) = selection.parse::<usize>() {
                    if num < menu_items.len() {
                        let selected = &menu_items[num];
                        match selected.item_type {
                            GopherItem::Directory => {
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
                                stream
                                    .write_all(format!("{}\r\n", selected.selector).as_bytes())?;
                                let content = read_text_content(&mut stream)?;
                                println!("\n{}", content);
                                println!("\nPress Enter to continue...");
                                std::io::stdin().read_line(&mut String::new())?;
                            }
                            GopherItem::Info => {
                                println!("Info item - no action needed");
                            }
                        }
                    } else {
                        println!("Invalid selection!");
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
enum GopherItem {
    Text,
    Directory,
    Info,
}

#[derive(Debug)]
struct GopherUrl {
    host: String,
    // unsigned (non-negative) 16-bit integer (0 to 65535)
    port: u16,
    selector: String,
}

#[derive(Debug)]
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
    // fn display(&self, index: usize) -> String {
    //     let type_indicator = match self.item_type {
    //         GopherItem::Directory => "[DIR]",
    //         GopherItem::Text => "[TXT]",
    //         GopherItem::Info => "[INFO]",
    //     };
    //     format!("[{}] {} {}", index, type_indicator, self.display_text)
    // }

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
