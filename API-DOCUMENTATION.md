# Kasuki Bot API Documentation

## Overview

This API provides access to various data points from the Kasuki bot database. Authentication is required for all endpoints except the health check.

## Authentication

Authentication is performed using an API key that must be included in the `X-API-Key` header of all requests.

Example:
```
X-API-Key: your_secure_api_key_here
```

The API key is configured in the `config.toml` file under the `[api]` section.

## Endpoints

### Health Check

```
GET /health
```

Returns a 200 OK status code if the API is running. No authentication required.

### Anime Songs

```
GET /anime/songs?limit=20&offset=0
```

Returns a list of anime songs from the database.

**Query Parameters:**
- `limit` (optional): Maximum number of results to return (default: 20)
- `offset` (optional): Number of results to skip (default: 0)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "anilist_id": "123",
      "anime_en_name": "My Hero Academia",
      "anime_jp_name": "僕のヒーローアカデミア",
      "song_type": "op",
      "song_name": "The Day",
      "hq_url": "https://files.catbox.moe/abc123.webm",
      "mq_url": "https://files.catbox.moe/def456.webm",
      "audio_url": "https://files.catbox.moe/ghi789.mp3"
    },
    ...
  ],
  "error": null
}
```

### Random Stats

```
GET /stats/random
```

Returns the current random stats containing the last page numbers for anime and manga.

**Response:**
```json
{
  "success": true,
  "data": {
    "anime_last_page": 1234,
    "manga_last_page": 5678
  },
  "error": null
}
```

### Command Usage

```
GET /commands/usage
```

Returns the usage count for each command, aggregated across all users.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "command_name": "ping",
      "usage_count": 12345
    },
    {
      "command_name": "color",
      "usage_count": 6789
    },
    ...
  ],
  "error": null
}
```

### Command List

```
GET /commands/list
```

Returns a list of all commands available in the bot.

**Response:**
```json
{
  "success": true,
  "data": [
    "ping",
    "color",
    "profile",
    ...
  ],
  "error": null
}
```

### Ping Statistics

```
GET /stats/ping
```

Returns the latest ping information for each shard.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "shard_id": "0",
      "latency": "52",
      "timestamp": "2025-02-28T12:34:56.789"
    },
    {
      "shard_id": "1",
      "latency": "48",
      "timestamp": "2025-02-28T12:34:55.123"
    },
    ...
  ],
  "error": null
}
```

### User Count

```
GET /stats/users
```

Returns the total number of users in the database.

**Response:**
```json
{
  "success": true,
  "data": {
    "count": 123456
  },
  "error": null
}
```

### Guild Count

```
GET /stats/guilds
```

Returns the total number of guilds (servers) in the database.

**Response:**
```json
{
  "success": true,
  "data": {
    "count": 7890
  },
  "error": null
}
```

## Error Responses

If an error occurs, the API will return an appropriate HTTP status code along with an error message:

```json
{
  "success": false,
  "data": null,
  "error": "Error message"
}
```

Common error codes:
- `401 Unauthorized`: Invalid or missing API key
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server-side error

## Configuration

To enable the API, add the following to your `config.toml` file:

```toml
[api]
enabled = true
port = 8080
api_key = "your_secure_api_key_here"
```

Make sure to use a strong API key in production environments. The API will only be available if `enabled` is set to `true`.

## Usage Examples

### cURL

```bash
# Health check
curl -X GET http://localhost:8080/health

# Get anime songs
curl -X GET http://localhost:8080/anime/songs?limit=10 \
  -H "X-API-Key: your_secure_api_key_here"

# Get guild count
curl -X GET http://localhost:8080/stats/guilds \
  -H "X-API-Key: your_secure_api_key_here"
```

### Python

```python
import requests

API_URL = "http://localhost:8080"
API_KEY = "your_secure_api_key_here"

headers = {
    "X-API-Key": API_KEY
}

# Health check
response = requests.get(f"{API_URL}/health")
print(f"Health check: {response.status_code}")

# Get command usage
response = requests.get(f"{API_URL}/commands/usage", headers=headers)
if response.status_code == 200:
    data = response.json()
    for command in data["data"]:
        print(f"{command['command_name']}: {command['usage_count']} uses")
```

### JavaScript (Node.js)

```javascript
const axios = require('axios');

const API_URL = 'http://localhost:8080';
const API_KEY = 'your_secure_api_key_here';

const headers = {
  'X-API-Key': API_KEY
};

// Get ping statistics
axios.get(`${API_URL}/stats/ping`, { headers })
  .then(response => {
    const pingData = response.data.data;
    pingData.forEach(shard => {
      console.log(`Shard ${shard.shard_id}: ${shard.latency}ms at ${shard.timestamp}`);
    });
  })
  .catch(error => {
    console.error('Error fetching ping statistics:', error);
  });

// Get user and guild counts
async function getCounts() {
  try {
    const [usersResponse, guildsResponse] = await Promise.all([
      axios.get(`${API_URL}/stats/users`, { headers }),
      axios.get(`${API_URL}/stats/guilds`, { headers })
    ]);
    
    console.log(`Total users: ${usersResponse.data.data.count}`);
    console.log(`Total guilds: ${guildsResponse.data.data.count}`);
  } catch (error) {
    console.error('Error fetching counts:', error);
  }
}

getCounts();
