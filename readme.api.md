# Realm API Documentation

Realm supports HTTP API for managing instances with OpenAPI standard and API key authentication.

## Table of Contents

- [Quick Start](#quick-start)
- [API Authentication](#api-authentication)
- [API Endpoints](#api-endpoints)
- [Complete API Usage Examples](#complete-api-usage-examples)
- [Instance Management Examples](#instance-management-examples)
- [Advanced Configuration Examples](#advanced-configuration-examples)
- [Security Best Practices](#security-best-practices)
- [Error Handling](#error-handling)

## Quick Start

### Start API Server

```shell
# Start without authentication (not recommended for production)
./target/release/realm api --port 8080

# Start with API key authentication (recommended)
./target/release/realm api --port 8080 --api-key "your-secure-api-key-here"
```

### Basic Usage

```bash
# Create a simple proxy instance
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:1080",
    "remote": "example.com:80"
  }'

# List all instances
curl -H "X-API-Key: your-secure-api-key-here" \
     http://localhost:8080/instances
```

## API Authentication

The API uses API key authentication via the `X-API-Key` header. If no API key is configured, authentication is disabled (not recommended for production environments).

### Authentication Headers

```bash
curl -H "X-API-Key: your-secure-api-key-here" \
     http://localhost:8080/instances
```

### Generate Secure API Key

```bash
# Generate a 32-byte hex string
openssl rand -hex 32

# Or use /dev/urandom
head -c 32 /dev/urandom | xxd -p -c 32
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/instances` | List all instances |
| POST | `/instances` | Create a new instance |
| GET | `/instances/{id}` | Get instance details |
| PUT | `/instances/{id}` | Update instance configuration |
| DELETE | `/instances/{id}` | Delete instance |
| POST | `/instances/{id}/start` | Start a stopped instance |
| POST | `/instances/{id}/stop` | Stop a running instance |
| POST | `/instances/{id}/restart` | Restart an instance |

### OpenAPI Documentation

Visit `http://localhost:8080/swagger-ui` for interactive API documentation.

## Complete API Usage Examples

### 1. Basic TCP Proxy Instance

Create a simple TCP proxy instance:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:1080",
    "remote": "example.com:80"
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "config": {
    "listen": "127.0.0.1:1080",
    "remote": "example.com:80"
  },
  "status": "Running"
}
```

### 2. UDP and TCP Proxy Instance

Create an instance that handles both TCP and UDP traffic:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "0.0.0.0:53",
    "remote": "8.8.8.8:53",
    "network": {
      "use_udp": true
    }
  }'
```

### 3. Load Balancing Instance

Create an instance with load balancing across multiple backends:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8080",
    "remote": "backend1.example.com:80",
    "extra_remotes": [
      "backend2.example.com:80",
      "backend3.example.com:80"
    ],
    "balance": "roundrobin: 3, 2, 1"
  }'
```

### 4. Instance with Network Configuration

Create an instance with custom network settings:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8443",
    "remote": "secure.example.com:443",
    "network": {
      "tcp_timeout": 10,
      "udp_timeout": 60,
      "tcp_keepalive": 30,
      "tcp_keepalive_probe": 5
    }
  }'
```

### 5. Instance with Interface Binding

Create an instance bound to specific network interfaces:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "192.168.1.100:8080",
    "remote": "api.example.com:443",
    "interface": "eth0",
    "listen_interface": "eth0"
  }'
```

### 6. Instance with Through Configuration

Create an instance that sends traffic through a specific IP:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:3128",
    "remote": "proxy.example.com:8080",
    "through": "10.0.0.1"
  }'
```

### 7. Instance with Transport Layer Security

Create an instance with TLS/SSL support:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8443",
    "remote": "secure.example.com:443",
    "listen_transport": "tls:cert.pem,key.pem",
    "remote_transport": "tls:ca.pem"
  }'
```

### 8. Instance with WebSocket Support

Create an instance with WebSocket transport:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8080",
    "remote": "ws.example.com:80",
    "remote_transport": "ws:/websocket"
  }'
```

### 9. Instance with Proxy Protocol

Create an instance with proxy protocol support:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8080",
    "remote": "backend.example.com:80",
    "network": {
      "send_proxy": true,
      "send_proxy_version": 2,
      "accept_proxy": false
    }
  }'
```

### 10. MPTCP Enabled Instance

Create an instance with MPTCP support:

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8080",
    "remote": "mptcp.example.com:80",
    "network": {
      "send_mptcp": true,
      "accept_mptcp": true
    }
  }'
```

## Instance Management Examples

### List All Instances

```bash
curl -H "X-API-Key: your-secure-api-key-here" \
     http://localhost:8080/instances
```

Response:
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "config": {
      "listen": "127.0.0.1:1080",
      "remote": "example.com:80"
    },
    "status": "Running"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "config": {
      "listen": "127.0.0.1:8080",
      "remote": "api.example.com:443"
    },
    "status": "Running"
  }
]
```

### Get Specific Instance

```bash
curl -H "X-API-Key: your-secure-api-key-here" \
     http://localhost:8080/instances/550e8400-e29b-41d4-a716-446655440000
```

### Update Instance Configuration

```bash
curl -X PUT http://localhost:8080/instances/550e8400-e29b-41d4-a716-446655440000 \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:1080",
    "remote": "updated.example.com:80",
    "network": {
      "tcp_timeout": 15
    }
  }'
```

### Start Instance

```bash
curl -X POST http://localhost:8080/instances/550e8400-e29b-41d4-a716-446655440000/start \
  -H "X-API-Key: your-secure-api-key-here"
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "config": {
    "listen": "127.0.0.1:1080",
    "remote": "example.com:80"
  },
  "status": "Running"
}
```

### Stop Instance

```bash
curl -X POST http://localhost:8080/instances/550e8400-e29b-41d4-a716-446655440000/stop \
  -H "X-API-Key: your-secure-api-key-here"
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "config": {
    "listen": "127.0.0.1:1080",
    "remote": "example.com:80"
  },
  "status": "Stopped"
}
```

### Restart Instance

```bash
curl -X POST http://localhost:8080/instances/550e8400-e29b-41d4-a716-446655440000/restart \
  -H "X-API-Key: your-secure-api-key-here"
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "config": {
    "listen": "127.0.0.1:1080",
    "remote": "example.com:80"
  },
  "status": "Running"
}
```

### Delete Instance

```bash
curl -X DELETE http://localhost:8080/instances/550e8400-e29b-41d4-a716-446655440000 \
  -H "X-API-Key: your-secure-api-key-here"
```

## Advanced Configuration Examples

### High-Performance Instance

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "0.0.0.0:443",
    "remote": "backend.example.com:443",
    "network": {
      "tcp_timeout": 3,
      "tcp_keepalive": 10,
      "tcp_keepalive_probe": 3,
      "send_mptcp": true,
      "accept_mptcp": true
    }
  }'
```

### IPv6-Only Instance

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "[::1]:8080",
    "remote": "[2001:db8::1]:80",
    "network": {
      "ipv6_only": true
    }
  }'
```

### Complex Load Balancing Setup

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-secure-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:80",
    "remote": "web1.example.com:80",
    "extra_remotes": [
      "web2.example.com:80",
      "web3.example.com:80",
      "web4.example.com:80"
    ],
    "balance": "iphash: 1, 1, 1, 1",
    "network": {
      "tcp_timeout": 5,
      "tcp_keepalive": 15
    }
  }'
```

## Security Best Practices

### 1. Always Use API Key Authentication

```bash
# Generate a secure API key
openssl rand -hex 32

# Start server with authentication
./realm api --port 8080 --api-key "your-generated-secure-key"
```

### 2. Use HTTPS in Production

Configure a reverse proxy (nginx, caddy, etc.) with TLS termination:

```nginx
server {
    listen 443 ssl;
    server_name your-api.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 3. Network Security

- Bind API server to localhost or internal network only
- Use firewall rules to restrict access
- Implement rate limiting

### 4. API Key Management

- Rotate API keys regularly
- Use different keys for different clients/applications
- Store API keys securely (environment variables, secret management systems)

## Error Handling

The API returns appropriate HTTP status codes:

- `200` - Success
- `201` - Instance created
- `400` - Invalid configuration
- `401` - Authentication failed
- `404` - Instance not found
- `409` - Instance already in requested state
- `500` - Internal server error

Error response examples:

```json
{
  "error": "Invalid listen address format"
}
```

```json
{
  "error": "Authentication required"
}
```

```json
{
  "error": "Instance already running"
}
```

```json
{
  "error": "Instance already stopped"
}
```

---

For more information about Realm configuration options, see the main [README.md](README.md).
