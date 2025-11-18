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

3. Build the application:
   ```bash
   trunk build --release
   ```

### Configuration Options

- `KASUKI_API_URL`: The base URL of the Kasuki API server (required)
  - For local development: `http://localhost:8080`
  - For production: `https://your-domain.com`

### Notes

- The `.env` file is loaded at **compile time**, not runtime
- Changes to `.env` require rebuilding the application
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
