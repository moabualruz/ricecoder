# Project Overview

## Purpose
This crate provides comprehensive activity logging, audit trails, and monitoring capabilities for RiceCoder operations. It enables structured logging of user actions, system events, and compliance-related activities.

## Key Features
- Structured Event Logging with hierarchical severity levels
- Audit Trails for immutable compliance records
- Session Activity Tracking
- Performance Monitoring
- Compliance Logging for enterprise use

## Architecture
- High Performance: Async logging with minimal overhead
- Structured Data: JSON-based logging
- Multiple Outputs: Console, file, database, external services
- Filtering & Search capabilities
- Configurable retention policies

## Modules
- audit: Audit logging and compliance
- error: Error handling types
- events: Activity event definitions
- logger: Main logging interface
- monitoring: Performance metrics
- session_tracking: User session monitoring
- storage: Log storage and retention