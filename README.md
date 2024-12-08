# Gopher Client

A terminal-based Gopher protocol client written in Rust that allows interactive navigation of Gopherspace.

## Features

- Interactive terminal-based navigation
- Support for multiple Gopher item types:
  - Text files (0)
  - Directories/Menus (1)
  - Search items (7)
  - Image files (g, I)
  - Information messages (i)
- File download capability for binary content
- Automatic file opening with system default applications
- Navigation history with back functionality
- Menu reloading
- Search functionality

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Building from source

1. Clone the repository:
```bash
git clone <your-repository-url>
cd gopher-client
```

2. Build the project:
```bash
cargo build --release
```

The compiled binary will be available at `target/release/gopher_client`

## Usage

Run the client:
```bash
cargo run
```

### Navigation

- Use numbers `[0-9]` to select menu items
- `B` to go back to the previous menu
- `R` to reload the current menu
- `Q` to quit the program

### Binary File Handling

When selecting binary files (like images), you have the following options:
1. Download the file
2. Download and open with the default system program
3. Cancel the operation

## Server Compatibility

The client has been tested with several Gopher servers including:
- gopher://tramberend.de
- gopher://gopher.floodgap.com
- gopher://gopher.quux.org

## Project Structure

```
gopher-client/
├── src/
│   ├── main.rs           # Application entry point
│   ├── lib.rs            # Library exports
│   ├── models.rs         # Data structures and types
│   ├── handlers.rs       # Request handlers
│   ├── ui.rs             # User interface functions
│   └── utils.rs          # Utility functions
├── Cargo.toml            # Project configuration
└── README.md            # This file
```
