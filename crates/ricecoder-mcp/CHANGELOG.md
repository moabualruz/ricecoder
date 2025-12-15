# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-12-09

### Added

- Initial release

### Changed

- N/A

### Fixed

- N/A

### Removed

- N/A

### Deprecated

- N/A

### Security

- N/A

## [0.1.1] - 2025-12-15

### Added

- **MCP Protocol 2025-06-18 Support**: Updated to latest MCP specification with enterprise error codes (-32005 to -32010)
- **Audit Logging Integration**: Comprehensive audit logging for all MCP operations including server management, tool execution, and security events
- **Enterprise Security Features**: OAuth 2.0 integration for HTTP transports, enhanced permission checking with audit trails
- **RBAC Integration**: Role-Based Access Control for MCP server and tool access with enterprise security integration
- **Compliance Monitoring**: SOC 2, GDPR, and HIPAA compliance monitoring with violation tracking and reporting
- **Enterprise Monitoring**: Comprehensive monitoring and metrics for MCP operations with health status assessment
- **Enhanced Connection Pooling**: Advanced connection pool management with enterprise monitoring and statistics
- **Failover Mechanisms**: Automatic server failover with exponential backoff and health-based routing
- **Health Monitoring**: Enterprise-grade health monitoring with configurable thresholds and alerting
- **Tool Enablement/Disablement**: Per-server tool enablement/disablement with permission controls and audit logging
- **Security Compliance**: SOC 2/GDPR/HIPAA compliance features with audit trails and compliance reporting

### Changed

- Updated MCP error codes to include enterprise-specific codes
- Enhanced transport layer with OAuth 2.0 support
- Improved server management with audit logging integration
- Updated tool execution with comprehensive audit trails

### Security

- Added enterprise audit logging for all security-relevant operations
- Implemented OAuth 2.0 authentication for HTTP transports
- Enhanced permission checking with detailed audit trails
- Added compliance reporting for enterprise security requirements
