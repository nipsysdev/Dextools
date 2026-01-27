# storeman - A simple Logos Storage GUI node

storeman is a Desktop & Android app for connecting, uploading and downloading files from Logos Storage.

## Getting Started

### Prerequisites

- Node.js (v24 or higher)
- Rust and Cargo
- Tauri CLI

### Installation

1. Clone the repository
2. Install dependencies:
   ```bash
   npm install
   ```

### Development

Run the development server:

```bash
cargo tauri dev
```

### CLI Arguments

The application supports command-line arguments for providing custom configurations.

**Available Options:**

- `-p, --port <VALUE>` - Custom port for the storage node (default: 8089)
- `-d, --data-dir <VALUE>` - Custom data directory for storage
- `-h, --help` - Display help message and exit

**Examples:**

```bash
# Run with custom port
./storeman --port 8090

# Run with custom data directory
./storeman --data-dir /path/to/custom/data

# Run with both custom port and data directory
./storeman --port 8090 --data-dir /path/to/custom/data

# Display help
./storeman --help
```

**Note:** When running with `cargo tauri dev`, pass arguments after `--`:

```bash
cargo tauri dev -- --port 8090 --data-dir /path/to/custom/data
```

This allows you to run multiple instances of the application simultaneously, each with its own port and data directory.

### Build

Build the production application:

```bash
cargo tauri build
```
