use anyhow::Result;
use crate::models::GopherUrl;
use std::io::{stdin, stdout, Read, Write};
use std::net::TcpStream;

pub fn read_text_content(stream: &mut TcpStream) -> Result<String> {
    let mut content = Vec::new();
    stream.read_to_end(&mut content)?;
    
    String::from_utf8(content)
        .or_else(|err| Ok(String::from_utf8_lossy(err.as_bytes()).into_owned()))
}

pub fn wait_for_enter() -> Result<()> {
    println!("Press Enter to continue...");
    stdin().read_line(&mut String::new())?;
    Ok(())
}

pub fn read_user_input() -> Result<String> {
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(input.trim().to_lowercase())
}

pub fn handle_error(error: &str) -> Result<()> {
    println!("\n{}", error);
    wait_for_enter()?;
    Ok(())
}

pub fn get_initial_url() -> Result<GopherUrl> {
    loop {
        println!("\n=== Gopher Client ===");
        println!("1. Connect to gopher://tramberend.de");
        println!("2. Connect to gopher://gopher.floodgap.com");
        println!("3. Connect to gopher://gopher.quux.org:70/ IMAGE TYPE TEST");
        println!("4. Enter custom Gopher URI");
        println!("q. Quit");
        print!("\nPlease choose: ");
        stdout().flush()?;

        match read_user_input()?.as_str() {
            "1" => return GopherUrl::parse("gopher://tramberend.de"),
            "2" => return GopherUrl::parse("gopher://gopher.floodgap.com"),
            "3" => return GopherUrl::parse("gopher://gopher.quux.org:70/"),
            "4" => {
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