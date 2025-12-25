# ricecoder-api

RESTful API for RiceCoder with session management and MCP tool execution.

## DDD Layer

**Layer**: Infrastructure (API Gateway)

### Responsibilities

- RESTful HTTP API endpoints
- Session management via HTTP
- MCP tool execution proxy
- Authentication and authorization
- Rate limiting and request validation

### SOLID Analysis

| Principle | Score | Notes |
|-----------|-------|-------|
| SRP | ✅ | Handlers, middleware, routes clearly separated |
| OCP | ✅ | Extensible via new handlers and middleware |
| LSP | ✅ | Consistent handler interfaces |
| ISP | ✅ | Segregated middleware concerns (auth, logging, rate limiting) |
| DIP | ✅ | Depends on session, MCP, security abstractions |

**Score**: 5/5

### Integration Points

| Component | Direction | Purpose |
|-----------|-----------|---------|
| ricecoder-sessions | Depends on | Session management |
| ricecoder-mcp | Depends on | MCP tool execution |
| ricecoder-security | Depends on | Authentication |
| ricecoder-di | Depends on | Dependency injection |
| ricecoder-activity-log | Depends on | Request logging |
| ricecoder-agents | Depends on | Agent execution |

## Features

- **Endpoints**: Sessions, providers, tools, health
- **Authentication**: JWT-based with bcrypt password hashing
- **Middleware**: CORS, logging, rate limiting
- **Documentation**: OpenAPI/Swagger via utoipa

## Usage

```bash
# Start API server
ricecoder-api --port 8080

# Or programmatically
use ricecoder_api::Server;
Server::new(config).run().await?;
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/api/v1/sessions` | POST | Create session |
| `/api/v1/sessions/{id}` | GET | Get session |
| `/api/v1/tools/execute` | POST | Execute MCP tool |

## License

MIT
