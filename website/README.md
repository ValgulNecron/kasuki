# Tauri + Leptos

This template should help get you started developing with Tauri and Leptos.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Getting Started

### Install Dependencies

Before running the project, you need to install some dependencies:

1. **Install `trunk`**:
   ```bash
   cargo install trunk
   ```

2. **Add `wasm32-unknown-unknown` target**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Install `webkit2gtk-4.1` (Linux only)**:
   If you are on an Arch Linux based system, run:
   ```bash
   sudo pacman -Syu
   sudo pacman -S webkit2gtk-4.1 xorg-server-xvfb
   ```
   If you are on a Debian/Ubuntu based system, run:
   ```bash
   sudo apt update
   sudo apt install -y libwebkit2gtk-4.1-dev xvfb
   ```
   For other Linux distributions, please refer to your distribution's documentation for installing `webkit2gtk-4.1` and
   `xvfb` or their equivalents.

### Run the Project

To run the project, use the following command:

```bash
xvfb-run cargo tauri dev
```

### Notes on Deprecation Warnings

You may encounter deprecation warnings related to `leptos::prelude::create_signal` and `leptos::prelude::create_effect`.
These functions are being renamed to `signal()` and `Effect::new()` respectively to conform to Rust idioms. The project
has been updated to use the new APIs.
