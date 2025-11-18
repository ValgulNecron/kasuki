# Kasuki Website Configuration

## Environment Variables

The website uses compile-time environment variables for configuration. Create a `.env` file in this directory to configure the application.

### Setup

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and set the API URL:
   ```
   KASUKI_API_URL=http://localhost:8080
   ```
   
   **Important:** Make sure there are no extra spaces and the line is not commented out (no `#` at the start).

3. **Clean and rebuild** the application:
   ```bash
   # Clean previous builds to ensure .env changes are picked up
   cargo clean
   trunk clean
   
   # Rebuild
   trunk build --release
   ```

### Configuration Options

- `KASUKI_API_URL`: The base URL of the Kasuki API server (required)
  - For local development: `http://localhost:8080`
  - For production: `https://your-domain.com`

### Troubleshooting

#### Changes to .env not taking effect

If you change the `.env` file and the changes aren't reflected:

1. **Clean the build cache:**
   ```bash
   cargo clean
   trunk clean
   ```

2. **Rebuild from scratch:**
   ```bash
   trunk build --release
   ```

3. **Verify the .env file format:**
   - Make sure `KASUKI_API_URL=http://localhost:8080` is not commented out
   - No spaces around the `=` sign
   - No quotes around the URL
   - File encoding should be UTF-8

4. **Check build output:**
   During `trunk build`, look for these messages:
   ```
   Loading .env from: ...
   Found KASUKI_API_URL in .env: http://localhost:8080
   Build-time KASUKI_API_URL: http://localhost:8080
   ```

#### Wrong port being used

If the login redirects to the wrong port (e.g., 8080 when you set 80):

1. The old build cache might still be in use - run `cargo clean` and `trunk clean`
2. Make sure the `.env` file is in the correct directory (`website/kasuki-website/.env`)
3. Verify the .env file content doesn't have typos or extra characters

### Notes

- The `.env` file is loaded at **compile time**, not runtime
- Changes to `.env` require **cleaning and rebuilding** the application
- The `.env` file is in `.gitignore` to prevent committing sensitive configuration
- Use `.env.example` as a template for deployment

## Discord OAuth Setup

To enable Discord OAuth login:

1. Create a Discord application at https://discord.com/developers/applications
2. Configure the OAuth2 redirect URI to match your API server callback URL
3. Update the bot's `config.toml` with your Discord Client ID and Secret
4. Set `api.enabled = true` in the bot configuration
5. Start the bot (which will also start the API server on the configured port)
6. Build and deploy this website with `KASUKI_API_URL` pointing to your API server

## Development

For development with hot reload:
```bash
trunk serve
```

The website will be available at http://localhost:1420 (configured in `Trunk.toml`).

**Note:** With `trunk serve`, you'll need to rebuild (`Ctrl+C` and `trunk serve` again) if you change the `.env` file, as it's loaded at compile time.
