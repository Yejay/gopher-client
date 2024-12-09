use anyhow::{anyhow, Result};
use std::io::Write;
use std::net::TcpStream;
use url::Url;

#[derive(Debug, Clone)]
pub enum GopherItem {
    Text,
    Directory,
    Info,
    Search,
    Image,
}

#[derive(Debug, Clone)]
pub struct GopherUrl {
    pub host: String,
    pub port: u16,
    pub selector: String,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub item_type: GopherItem,
    pub display_text: String,
    pub selector: String,
    pub host: String,
    pub port: u16,
}

impl GopherUrl {
    pub fn parse(uri: &str) -> Result<Self> {
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

    // pub fn parse(uri: &str) -> Result<Self> {
    //     let parsed = Url::parse(uri)?;
    //     if parsed.scheme() != "gopher" {
    //         return Err(anyhow!("Not a gopher URL: {}", uri));
    //     }

    //     let host = parsed
    //         .host_str()
    //         .ok_or_else(|| anyhow!("No host found"))?
    //         .to_string();
    //     let port = parsed.port().unwrap_or(70);
        
    //     // Keep the entire path including the type indicator
    //     let selector = if parsed.path() == "/" {
    //         String::new()
    //     } else {
    //         parsed.path().to_string()
    //     };

    //     Ok(Self { host, port, selector })
    // }

    pub fn connect(&self) -> Result<TcpStream> {
        let address = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(&address)
            .map_err(|e| anyhow!("Failed to connect to {}: {}", address, e))?;

        stream
            .write_all(format!("{}\r\n", self.selector).as_bytes())
            .map_err(|e| anyhow!("Failed to send selector: {}", e))?;

        Ok(stream)
    }
}

impl MenuItem {
    pub fn parse(line: &str) -> Result<Self> {
        if line.trim().is_empty() {
            return Err(anyhow!("Empty line"));
        }

        let item_type = match line.chars().next() {
            Some('0') => GopherItem::Text,
            Some('1') => GopherItem::Directory,
            Some('i') => GopherItem::Info,
            Some('7') => GopherItem::Search,
            Some('g') | Some('I') => GopherItem::Image,
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

    pub fn display(&self, index: Option<usize>) -> String {
        match self.item_type {
            GopherItem::Info => {
                format!("    {}", self.display_text)
            }
            _ => {
                let type_indicator = match self.item_type {
                    GopherItem::Directory => "[DIR]",
                    GopherItem::Text => "[TXT]",
                    GopherItem::Info => unreachable!(),
                    GopherItem::Search => "[SEARCH]",
                    GopherItem::Image => "[IMG]",
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
}