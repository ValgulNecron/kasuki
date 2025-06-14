# Kasuki Development Guidelines

This document provides essential information for developers working on the Kasuki Discord bot project. It covers build
instructions, testing procedures, and development practices specific to this project.

## Build/Configuration Instructions

### Prerequisites

- Rust toolchain (latest stable version recommended)
- PostgreSQL database (for production)
- Discord bot token

### Setup Process

1. **Clone the repository**:
   ```bash
   git clone https://github.com/ValgulNecron/kasuki.git
   cd kasuki/bot
   ```

2. **Configure the bot**:
   Create or modify the `config.toml` file with the following essential sections:

   ```toml
   [bot]
   discord_token = "YOUR_DISCORD_TOKEN"
   bot_activity = "Let you get info from anilist."
   remove_old_commands = false
   respect_premium = true

   [db]
   db_type = "postgresql"  # or "sqlite" for development
   host = "localhost"
   port = 5432
   user = "username"
   password = "password"
   database = "kasuki"
   ```

3. **GraphQL Schema**:
   The project uses GraphQL for Anilist API interactions. The schema is automatically registered during build from
   `schemas/anilist.graphql`.

4. **Build the project**:
   ```bash
   cargo build
   ```

   For production builds:
   ```bash
   cargo build --release
   ```

5. **Run the bot**:
   ```bash
   cargo run
   ```

### Docker Deployment

The project includes Docker support for containerized deployment:

```bash
docker build -t kasuki .
docker run -v ./config.toml:/app/config.toml kasuki
```

## Testing Information

### Running Tests

The project uses Rust's built-in testing framework. To run all tests:

```bash
cargo test
```

To run tests for a specific module:

```bash
cargo test <module_path>
```

For example:

```bash
cargo test helper::string_utils
```

### Adding New Tests

1. **Unit Tests**: Add tests within the module file using the `#[cfg(test)]` attribute:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_your_function() {
           // Test implementation
           assert_eq!(your_function(input), expected_output);
       }
   }
   ```

2. **Test Example**:
   Here's an example of a simple test for a string utility function:

   ```rust
   // In src/helper/string_utils.rs
   pub fn capitalize_first(s: &str) -> String {
       if s.is_empty() {
           return String::new();
       }
       
       let mut chars = s.chars();
       match chars.next() {
           None => String::new(),
           Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_capitalize_first() {
           assert_eq!(capitalize_first("hello"), "Hello");
           assert_eq!(capitalize_first("world"), "World");
           assert_eq!(capitalize_first(""), "");
           assert_eq!(capitalize_first("a"), "A");
           assert_eq!(capitalize_first("ALREADY_CAPITALIZED"), "ALREADY_CAPITALIZED");
       }
   }
   ```

3. **Adding the Module**: When adding a new module with tests, make sure to update the appropriate `mod.rs` file to
   include your new module.

## Additional Development Information

### Code Style

The project uses a specific code style defined in `rustfmt.toml`:

- Hard tabs (not spaces)
- Same-line braces
- Unix-style newlines
- Compressed function arguments layout

Always run `rustfmt` before committing:

```bash
cargo fmt
```

Use `clippy` to catch common mistakes and improve code quality:

```bash
cargo clippy
```

### Documentation Standards

- Add docstrings to all public functions (except for autocomplete, run, and register functions)
- Document complex code sections with inline comments
- Follow Rust's standard documentation format with examples where appropriate

Example:

```rust
/// Capitalizes the first letter of a string.
///
/// # Arguments
///
/// * `s` - The string to capitalize
///
/// # Returns
///
/// A new string with the first letter capitalized
///
/// # Examples
///
/// ```
/// let result = capitalize_first("hello");
/// assert_eq!(result, "Hello");
/// ```
pub fn capitalize_first(s: &str) -> String {
    // Implementation
}
```

### Project Structure

- **src/command/**: Discord bot commands organized by category
- **src/helper/**: Utility functions and helpers
- **src/background_task/**: Background processes
- **src/database/**: Database interaction code
- **src/structure/**: Data structures and models

### GraphQL Integration

The project uses the `cynic` crate for GraphQL integration with Anilist. When modifying GraphQL queries:

1. Update the query in the appropriate file
2. The build script will automatically register the schema

### Contribution Workflow

1. Create a feature branch
2. Implement changes following the code style guidelines
3. Add tests for new functionality
4. Run tests to ensure everything works
5. Document changes in your commit message, including performance impacts
6. Submit a pull request

When modifying existing functionality, clearly document what has changed and the impact (performance, readability, API
version changes, etc.).