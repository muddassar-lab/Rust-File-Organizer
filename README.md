# File Organizer

<div align="center">

![File Organizer Logo](assets/logo.png)

[![Release](https://github.com/yourusername/file-organizer/actions/workflows/release.yml/badge.svg)](https://github.com/yourusername/file-organizer/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A powerful, cross-platform file organization tool built in Rust that helps you keep your directories clean and organized.

[Installation](#installation) •
[Features](#features) •
[Usage](#usage) •
[Contributing](#contributing)

</div>

## Features

- 📁 **Smart File Organization**: Automatically organizes files based on type, date, or custom rules
- 🎯 **Multiple Organization Modes**:
  - By file type (images, documents, videos, etc.)
  - By date (year/month/day)
  - By custom rules and patterns
- 💫 **Interactive TUI**: Beautiful terminal user interface with real-time progress
- ⚡ **Blazingly Fast**: Built in Rust for maximum performance
- 🔄 **Resume Support**: Can resume interrupted operations
- 🔍 **Preview Mode**: See how files will be organized before making changes
- 🔒 **Safe Operations**: Never overwrites existing files without permission
- 📊 **Detailed Statistics**: Get insights about your file organization
- 🌐 **Cross-Platform**: Works on Windows, macOS, and Linux

## Installation

### From Releases

Download the latest installer for your platform from our [releases page](https://github.com/yourusername/file-organizer/releases).

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/file-organizer.git
cd file-organizer

# Build and install
cargo install --path .
```

## Usage

### Basic Usage

1. Launch the application:
```bash
file-organizer
```

2. Select your source directory
3. Choose your destination directory
4. Select organization mode
5. Review and confirm

### Command Line Options

```bash
file-organizer [OPTIONS] [SOURCE_DIR] [DEST_DIR]

Options:
  -t, --type     Organize by file type
  -d, --date     Organize by date
  -p, --preview  Preview mode (no changes made)
  -r, --resume   Resume previous operation
  -h, --help     Show help information
```

### Organization Modes

#### By File Type
Files are organized into directories based on their type:
```
Documents/
├── Images/
├── Documents/
├── Videos/
├── Music/
└── Others/
```

#### By Date
Files are organized by their creation/modification date:
```
Documents/
├── 2024/
│   ├── January/
│   └── February/
└── 2023/
    ├── December/
    └── November/
```

## Development

### Prerequisites

- Rust 1.70 or higher
- Cargo
- (Windows only) WiX Toolset for MSI creation
- (Optional) cargo-release for release management

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Running Tests

```bash
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'feat: Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Rust](https://www.rust-lang.org/) - For the amazing programming language
- [Ratatui](https://github.com/ratatui-org/ratatui) - For the terminal user interface
- [Crossterm](https://github.com/crossterm-rs/crossterm) - For cross-platform terminal manipulation
- All our [contributors](https://github.com/yourusername/file-organizer/graphs/contributors)