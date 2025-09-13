# Realm HTTP API Documentation

Realm provides HTTP API for dynamic instance management with two deployment modes: basic mode for simple use cases and hybrid mode for enterprise deployments with global configuration management.

## Table of Contents

- [Quick Start](#quick-start)
- [Deployment Modes](#deployment-modes)
- [API Authentication](#api-authentication)
- [Global Configuration Architecture](#global-configuration-architecture)
- [API Reference](#api-reference)
- [Instance Configuration Fields](#instance-configuration-fields)
- [Usage Examples](#usage-examples)
- [Best Practices](#best-practices)
- [Error Handling](#error-handling)

## Quick Start

### Start API Server

```bash
# Default global configuration with authentication
export REALM_API_KEY="your-api-key"
realm api --port 8080

# Custom global configuration with authentication
export REALM_API_KEY="your-api-key"
realm api -c config.json --port 8080
```

### Create First Proxy Instance

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8080",
    "remote": "httpbin.org:80"
  }'
```

## Deployment Modes

### Basic Mode

```bash
export REALM_API_KEY="your-api-key"
realm api --port 8080
```

**Use Cases:**
- Development and testing environments
- Simple proxy scenarios
- Standalone instance deployments

**Features:**
- Each instance uses default log/DNS settings
- No shared configuration between instances
- Minimal resource usage

### Hybrid Mode (Recommended)

```bash
export REALM_API_KEY="your-api-key"
realm api -c global-config.json --port 8080
```

**Use Cases:**
- Production environments
- Enterprise applications
- Centralized management requirements

**Features:**
- Global shared logging configuration
- Centralized DNS settings and caching
- Default network configuration inheritance
- Better resource management and observability

## API Authentication

### Security Configuration

```bash
# Development mode (no authentication)
realm api

# Production mode with authentication
export REALM_API_KEY="your-secure-api-key"
realm api --port 8080
```

**Security Best Practices:**
- Use `REALM_API_KEY` environment variable for authentication
- Use strong, randomly generated API keys: `openssl rand -hex 32`
- Rotate API keys regularly
- Use HTTPS/TLS in production deployments

### Request Headers

When authentication is enabled, all requests must include the authentication header:

```bash
curl -H "X-API-Key: your-api-key" http://localhost:8080/instances
```

## Global Configuration Architecture

### Configuration Hierarchy

```
Global Configuration (Process Level)
├── log: Shared logging system
├── dns: Shared DNS resolution and caching
└── network: Default network settings
    │
    └── Instance Configuration (Endpoint Level)
        ├── Endpoint-specific settings
        ├── Network configuration overrides (optional)
        └── Transport configuration (optional)
```

### Configuration Inheritance Priority

1. **Instance-level network settings** - Explicit overrides
2. **Global-level network settings** - Inherited default values
3. **Built-in default values** - System defaults

### Global Configuration Example

```json
{
  "log": {
    "level": "info",
    "output": "/var/log/realm-api.log"
  },
  "dns": {
    "mode": "ipv4_then_ipv6",
    "nameservers": ["8.8.8.8:53", "1.1.1.1:53"],
    "timeout": 5,
    "cache_size": 256
  },
  "network": {
    "tcp_keepalive": 60,
    "tcp_timeout": 10,
    "udp_timeout": 30,
    "send_proxy": false,
    "accept_proxy": false
  },
  "endpoints": []
}
```

## API Reference

### Instance Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/instances` | List all instances |
| `POST` | `/instances` | Create new instance |
| `GET` | `/instances/{id}` | Get instance details |
| `PUT` | `/instances/{id}` | Update instance configuration |
| `DELETE` | `/instances/{id}` | Delete instance |

### Instance Control

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/instances/{id}/start` | Start instance |
| `POST` | `/instances/{id}/stop` | Stop instance |
| `POST` | `/instances/{id}/restart` | Restart instance |

### Documentation

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/swagger-ui` | Interactive API documentation |
| `GET` | `/api-docs/openapi.json` | OpenAPI specification |

## Instance Configuration Fields

### Complete Configuration Structure (21 Fields)

#### Required Fields (2)

```json
{
  "listen": "0.0.0.0:8080",        // Listen address and port
  "remote": "target.com:80"        // Target server address
}
```

#### Load Balancing Fields (2)

```json
{
  "extra_remotes": ["server2:80", "server3:80"],  // Additional servers
  "balance": "roundrobin: 3, 2, 1"                // Load balancing strategy
}
```

#### Network Interface Fields (3)

```json
{
  "through": "192.168.1.100",      // Outbound IP binding
  "interface": "eth0",             // Outbound network interface
  "listen_interface": "lo"         // Listen network interface
}
```

#### Transport Encryption Fields (2)

```json
{
  "listen_transport": "tls;servername=api.example.com;cert=/etc/ssl/cert.pem;key=/etc/ssl/key.pem",
  "remote_transport": "ws;host=backend.com;path=/tunnel;tls;sni=backend.com"
}
```

#### Network Protocol Fields (12)

```json
{
  "network": {
    "no_tcp": false,              // Disable TCP
    "use_udp": true,              // Enable UDP
    "ipv6_only": false,           // IPv6 only mode
    "send_mptcp": true,           // Send multipath TCP
    "accept_mptcp": true,         // Accept multipath TCP
    "tcp_timeout": 30,            // TCP connection timeout (seconds)
    "udp_timeout": 60,            // UDP connection timeout (seconds)
    "tcp_keepalive": 120,         // TCP keepalive interval (seconds)
    "tcp_keepalive_probe": 5,     // TCP keepalive probe count
    "send_proxy": true,           // Send Proxy Protocol
    "accept_proxy": true,         // Accept Proxy Protocol
    "send_proxy_version": 2,      // Proxy Protocol version
    "accept_proxy_timeout": 10    // Proxy Protocol timeout
  }
}
```

## Usage Examples

### 1. Simple HTTP Proxy

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8080",
    "remote": "httpbin.org:80"
  }'
```

### 2. Load Balanced Proxy

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "0.0.0.0:443",
    "remote": "web1.internal:443",
    "extra_remotes": ["web2.internal:443", "web3.internal:443"],
    "balance": "roundrobin: 3, 2, 1",
    "network": {
      "tcp_keepalive": 60,
      "send_proxy": true,
      "send_proxy_version": 2
    }
  }'
```

### 3. TLS Termination Proxy

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "0.0.0.0:443",
    "remote": "internal-app:8080",
    "listen_transport": "tls;servername=api.example.com;cert=/etc/ssl/cert.pem;key=/etc/ssl/key.pem"
  }'
```

### 4. WebSocket Tunnel

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:1080",
    "remote": "tunnel-server.example.com:443",
    "remote_transport": "ws;host=tunnel-server.example.com;path=/tunnel;tls;sni=tunnel-server.example.com"
  }'
```

### 5. Game Server Proxy

```bash
curl -X POST http://localhost:8080/instances \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "0.0.0.0:25565",
    "remote": "gameserver.internal:25565",
    "network": {
      "use_udp": true,
      "tcp_keepalive": 300,
      "udp_timeout": 180
    }
  }'
```

### 6. Instance Management Operations

```bash
# List all instances
curl -H "X-API-Key: your-api-key" http://localhost:8080/instances

# Get instance details
curl -H "X-API-Key: your-api-key" \
     http://localhost:8080/instances/{instance-id}

# Stop instance
curl -X POST -H "X-API-Key: your-api-key" \
     http://localhost:8080/instances/{instance-id}/stop

# Update instance configuration
curl -X PUT http://localhost:8080/instances/{instance-id} \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "listen": "127.0.0.1:8080",
    "remote": "new-backend.example.com:80"
  }'

# Delete instance
curl -X DELETE -H "X-API-Key: your-api-key" \
     http://localhost:8080/instances/{instance-id}
```

## Best Practices

### Production Deployment

#### 1. Use Hybrid Mode

```bash
# Create global configuration file
cat > /etc/realm/global.json << EOF
{
  "log": {
    "level": "info",
    "output": "/var/log/realm/api.log"
  },
  "dns": {
    "mode": "ipv4_then_ipv6",
    "nameservers": ["8.8.8.8:53", "1.1.1.1:53"],
    "cache_size": 512
  },
  "network": {
    "tcp_keepalive": 60,
    "tcp_timeout": 10
  },
  "endpoints": []
}
EOF

# Start API server
export REALM_API_KEY=$(openssl rand -hex 32)
realm api -c /etc/realm/global.json --port 8080
```

#### 2. Security Configuration

```bash
# Generate strong API key
export REALM_API_KEY=$(openssl rand -hex 32)

# Create dedicated user
useradd -r -s /bin/false realm

# Run with restricted permissions
sudo -u realm env REALM_API_KEY="${REALM_API_KEY}" \
  realm api -c /etc/realm/global.json --port 8080
```

#### 3. Reverse Proxy Configuration

```nginx
# nginx configuration example
server {
    listen 443 ssl;
    server_name api.realm.example.com;
    
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

### Container Deployment

#### Docker Compose

```yaml
version: '3.8'
services:
  realm-api:
    image: realm:latest
    ports:
      - "8080:8080"
    volumes:
      - ./global.json:/etc/realm/global.json:ro
      - ./logs:/var/log/realm
    environment:
      - REALM_API_KEY=${REALM_API_KEY}
    command: >
      realm api 
      -c /etc/realm/global.json 
      --port 8080
    restart: unless-stopped
```

#### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: realm-api
spec:
  replicas: 1
  selector:
    matchLabels:
      app: realm-api
  template:
    metadata:
      labels:
        app: realm-api
    spec:
      containers:
      - name: realm-api
        image: realm:latest
        ports:
        - containerPort: 8080
        env:
        - name: REALM_API_KEY
          valueFrom:
            secretKeyRef:
              name: realm-secrets
              key: api-key
        volumeMounts:
        - name: config
          mountPath: /etc/realm
        command:
          - realm
          - api
          - -c
          - /etc/realm/global.json
          - --port
          - "8080"
      volumes:
      - name: config
        configMap:
          name: realm-config
```

### Monitoring and Logging

```bash
# Monitor API server logs
tail -f /var/log/realm/api.log

# Monitor system resources
top -p $(pgrep realm)

# Check instance status
curl -s -H "X-API-Key: ${REALM_API_KEY}" \
     http://localhost:8080/instances | \
     jq '.[] | {id: .id, status: .status, listen: .config.listen}'
```

### Performance Optimization

#### DNS Configuration Optimization

```json
{
  "dns": {
    "mode": "ipv4_then_ipv6",
    "cache_size": 1024,
    "timeout": 3,
    "nameservers": ["8.8.8.8:53", "1.1.1.1:53"]
  }
}
```

#### Network Optimization

```json
{
  "network": {
    "tcp_keepalive": 60,
    "tcp_timeout": 10,
    "send_mptcp": true,
    "accept_mptcp": true
  }
}
```

#### System Limits

```bash
# Increase file descriptor limit
ulimit -n 65536

# Increase network connection queue
sysctl net.core.somaxconn=65536
```

## Error Handling

### HTTP Status Codes

| Status Code | Meaning | Description |
|--------|------|------|
| `200` | Success | 请求Success处理 |
| `201` | Created | 实例创建Success |
| `204` | No Content | 实例删除Success |
| `400` | Bad Request | Invalid JSON or configuration |
| `401` | Unauthorized | Missing or invalid API key |
| `404` | Not Found | Instance ID does not exist |
| `409` | Conflict | Instance already in requested state |
| `500` | Server Error | Internal configuration or system error |

### Common Error Responses

#### Configuration Error

```json
{
  "error": "Invalid configuration",
  "details": "listen address already in use"
}
```

#### Authentication Error

```json
{
  "error": "Unauthorized",
  "details": "Invalid or missing API key"
}
```

#### Instance Not Found

```json
{
  "error": "Instance not found",
  "details": "No instance with ID 12345678-1234-5678-9abc-123456789abc"
}
```

### Troubleshooting

#### Common Issues and Solutions

**1. Port Already in Use**

```bash
# Check port usage
lsof -i :8080

# Terminate process if needed
kill $(lsof -t -i :8080)
```

**2. Permission Denied**

```bash
# Ports < 1024 require root privileges
sudo realm api --port 80 --api-key "key"

# Or use higher port number
realm api --port 8080 --api-key "key"
```

**3. Configuration File Issues**

```bash
# Check if file exists
ls -la /path/to/config.json

# Verify JSON format
jq . /path/to/config.json
```

**4. Instance Creation Failed**

```bash
# View detailed logs
tail -f /var/log/realm/api.log

# Check target server connectivity
telnet target-server.com 80
```

### Debug Mode

Enable detailed logging for debugging:

```json
{
  "log": {
    "level": "debug",
    "output": "/var/log/realm/debug.log"
  }
}
```

---

📖 **Related Documentation:** [Main Documentation](readme.md) | [Configuration Examples](examples/) | [Deployment Guide](readme.container.md)