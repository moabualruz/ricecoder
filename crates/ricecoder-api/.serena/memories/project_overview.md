# Project Overview

## Purpose
This is the `ricecoder-api` crate, part of the RiceCoder project. It provides a RESTful API server for RiceCoder, offering session management, MCP (Model Context Protocol) tool execution, and enterprise features with comprehensive authentication and monitoring.

## Tech Stack
- **Language**: Rust (2021 edition)
- **Web Framework**: Axum
- **HTTP Middleware**: Tower (with CORS, file serving, tracing)
- **Serialization**: Serde (JSON, YAML)
- **OpenAPI Documentation**: Utoipa with Swagger UI
- **Authentication**: JSON Web Tokens (jsonwebtoken), bcrypt for password hashing
- **Async Runtime**: Tokio
- **Error Handling**: Thiserror
- **Logging**: Tracing
- **Dependency Injection**: Custom DI container (ricecoder-di)
- **Session Management**: ricecoder-sessions
- **MCP Tools**: ricecoder-mcp
- **Security**: ricecoder-security
- **Activity Logging**: ricecoder-activity-log
- **Agents**: ricecoder-agents

## Architecture
The API is structured as a library crate with a binary entry point. Key modules include:
- `handlers/`: Route handlers for different endpoints (auth, health, providers, sessions, tools)
- `middleware/`: HTTP middleware for authentication, logging, rate limiting
- `models/`: Request/response data structures with OpenAPI schemas
- `routes/`: Route configuration
- `server/`: Server setup and startup
- `state/`: Application state management
- `error/`: Error types and handling

The server runs on `127.0.0.1:3000` by default.

## Key Features
- Session management for RiceCoder interactions
- MCP tool execution endpoints
- Authentication and authorization
- Health checks and load testing
- OpenAPI/Swagger documentation
- Comprehensive logging and monitoring