# Spreadsheet Application

[![Rust](https://github.com/Sourav-pi/Graphing-Spreadsheet-Software/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/Sourav-pi/Graphing-Spreadsheet-Software/actions/workflows/rust.yml)

A feature-rich spreadsheet application built in Rust that supports both GUI and CLI interfaces.

## Prerequisites

- **Rust and Cargo** (1.85+ required for the Rust 2024 edition)
- **For GUI on Linux**: Install the required dependencies:

  ```bash
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
  ```

- **For macOS and Windows**: No additional dependencies required

## Usage

| Command | Description |
|---------|-------------|
| `make ext1` | Run the GUI application |
| `make && ./target/release/spreadsheet <rows> <cols>` | Run the CLI application with specified rows and columns |
| `make docs` | Generate documentation |
| `make test` | Run all tests |
| `make coverage` | Check test coverage |

## Quick Start

### GUI Application

```bash
make ext1
```

### CLI Application

```bash
make
./target/release/spreadsheet 20 30  # Creates a 20x30 spreadsheet
```

### Running Tests

```bash
make test
```

## Features

- Interactive spreadsheet with formula support
- Data visualization with charts
- Both GUI and CLI interfaces
- File operations (open/save)
- Statistical functions

## Documentation

Generate and view the documentation:

```bash
make docs
```