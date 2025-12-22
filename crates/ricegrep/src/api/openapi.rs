use serde_json::{json, Value};

pub type OpenApiDocument = Value;

pub fn openapi_document() -> OpenApiDocument {
    json!({
        "openapi": "3.0.1",
        "info": {
            "title": "Ricegrep Hybrid Search API",
            "version": "0.1",
            "description": "REST surface for search operations backed by the hybrid search engine."
        },
        "paths": {
            "/search": {
                "post": {
                    "summary": "Execute a search request",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/SearchRequest"}
                            }
                        },
                        "required": true
                    },
                    "responses": {
                        "200": {
                            "description": "Search execution result",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/SearchResponse"}
                                }
                            }
                        }
                    }
                }
            },
            "/health": {
                "get": {
                    "summary": "Health check",
                    "responses": {
                        "200": {
                            "description": "Healthy",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/HealthStatus"}
                                }
                            }
                        }
                    }
                }
            },
            "/alerts": {
                "get": {
                    "summary": "Active alerts",
                    "responses": {
                        "200": {
                            "description": "Alert summary",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/AlertSummary"}
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/alerts/{name}/ack": {
                "post": {
                    "summary": "Acknowledge an alert",
                    "parameters": [
                        {
                            "name": "name",
                            "in": "path",
                            "required": true,
                            "schema": {"type": "string"}
                        }
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AlertActionRequest"}
                            }
                        },
                        "required": true
                    },
                    "responses": {
                        "200": {
                            "description": "Updated alert summary",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/AlertSummary"}
                                }
                            }
                        }
                    }
                }
            },
            "/alerts/{name}/resolve": {
                "post": {
                    "summary": "Resolve an alert with a note",
                    "parameters": [
                        {
                            "name": "name",
                            "in": "path",
                            "required": true,
                            "schema": {"type": "string"}
                        }
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/AlertActionRequest"}
                            }
                        },
                        "required": true
                    },
                    "responses": {
                        "200": {
                            "description": "Resolved alert",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/AlertSummary"}
                                }
                            }
                        }
                    }
                }
            },
            "/graphql": {
                "post": {
                    "summary": "GraphQL entrypoint",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/GraphQLRequest"}
                            }
                        },
                        "required": true
                    },
                    "responses": {
                        "200": {
                            "description": "GraphQL response",
                            "content": {
                                "application/json": {
                                    "schema": {"type": "object"}
                                }
                            }
                        }
                    }
                }
            },
            "/metrics/history": {
                "get": {
                    "summary": "Aggregated metrics history",
                    "parameters": [
                        {
                            "name": "minutes",
                            "in": "query",
                            "schema": {
                                "type": "integer",
                                "minimum": 1,
                                "description": "History window in minutes (max 129600)"
                            }
                        },
                        {
                            "name": "bucket_minutes",
                            "in": "query",
                            "schema": {
                                "type": "integer",
                                "minimum": 1,
                                "maximum": 60,
                                "description": "Aggregation bucket size in minutes"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Time bucketed metrics history entries",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {
                                            "$ref": "#/components/schemas/MetricsHistoryEntry"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "SearchRequest": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "limit": {"type": "integer", "minimum": 1},
                        "filters": {"$ref": "#/components/schemas/SearchFilters"},
                        "ranking": {"$ref": "#/components/schemas/RankingConfig"},
                        "timeout_ms": {"type": "integer", "minimum": 0}
                    },
                    "required": ["query"]
                },
                "SearchResponse": {
                    "type": "object",
                    "properties": {
                        "results": {
                            "type": "array",
                            "items": {"$ref": "#/components/schemas/SearchResult"}
                        },
                        "total_found": {"type": "integer", "minimum": 0},
                        "query_time_ms": {"type": "number"},
                        "request_id": {"type": "string"}
                    }
                },
                "SearchFilters": {
                    "type": "object",
                    "properties": {
                        "repository_id": {"type": "integer"},
                        "language": {"type": "string"},
                        "file_path_pattern": {"type": "string"}
                    }
                },
                "RankingConfig": {
                    "type": "object",
                    "properties": {
                        "lexical_weight": {"type": "number"},
                        "semantic_weight": {"type": "number"},
                        "rrf_k": {"type": "integer", "minimum": 1}
                    }
                },
                "SearchResult": {
                    "type": "object",
                    "properties": {
                        "chunk_id": {"type": "integer"},
                        "score": {"type": "number"},
                        "content": {"type": "string"},
                        "metadata": {"$ref": "#/components/schemas/ChunkMetadata"},
                        "highlights": {"type": "array", "items": {"type": "string"}}
                    }
                },
                "ChunkMetadata": {
                    "type": "object",
                    "properties": {
                        "chunk_id": {"type": "integer"},
                        "repository_id": {"type": "integer"},
                        "file_path": {"type": "string"},
                        "language": {"type": "string"},
                        "start_line": {"type": "integer"},
                        "end_line": {"type": "integer"},
                        "token_count": {"type": "integer"},
                        "checksum": {"type": "string"}
                    }
                },
                "HealthStatus": {
                    "type": "object",
                    "properties": {
                        "healthy": {"type": "boolean"},
                        "message": {"type": "string"},
                        "alerts": {
                            "type": "array",
                            "items": {
                                "$ref": "#/components/schemas/AlertSummary"
                            }
                        }
                    }
                },
                "AlertSummary": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "severity": {"type": "string", "enum": ["Info", "Warning", "Critical"]},
                        "state": {
                            "type": "string",
                            "enum": ["Inactive", "Pending", "Firing", "Resolved"]
                        },
                        "metric": {"type": "string"},
                        "value": {"type": "number", "nullable": true},
                        "threshold": {"type": "number", "nullable": true},
                        "description": {"type": "string"},
                        "acknowledged": {"type": "boolean"},
                        "acknowledged_by": {"type": "string", "nullable": true},
                        "acknowledged_at": {
                            "type": "string",
                            "format": "date-time",
                            "nullable": true
                        },
                        "resolved_at": {
                            "type": "string",
                            "format": "date-time",
                            "nullable": true
                        },
                        "resolution_note": {"type": "string", "nullable": true}
                    }
                },
                "AlertActionRequest": {
                    "type": "object",
                    "properties": {
                        "actor": {"type": "string"},
                        "note": {"type": "string"}
                    }
                },
                "MetricsHistoryEntry": {
                    "type": "object",
                    "properties": {
                        "timestamp": {
                            "type": "string",
                            "format": "date-time"
                        },
                        "avg_embedding_latency_ms": {
                            "type": "number",
                            "nullable": true
                        },
                        "avg_qdrant_latency_ms": {
                            "type": "number",
                            "nullable": true
                        },
                        "cache_miss_rate": {
                            "type": "number",
                            "nullable": true
                        },
                        "cpu_percent": {
                            "type": "number",
                            "nullable": true
                        },
                        "memory_usage_bytes": {
                            "type": "number",
                            "nullable": true
                        },
                        "disk_usage_bytes": {
                            "type": "number",
                            "nullable": true
                        },
                        "network_sent_bytes": {
                            "type": "number",
                            "nullable": true
                        },
                        "network_received_bytes": {
                            "type": "number",
                            "nullable": true
                        },
                        "index_size_bytes": {
                            "type": "number",
                            "nullable": true
                        },
                        "index_build_duration_seconds": {
                            "type": "number",
                            "nullable": true
                        }
                    }
                },
                "GraphQLRequest": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "variables": {"type": "object"}
                    },
                    "required": ["query"]
                }
            }
        }
    })
}
