//! MCP tool execution API handlers

use crate::{
    error::{ApiError, ApiResult},
    models::{ExecuteToolRequest, ExecuteToolResponse},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};

/// Execute MCP tool
#[utoipa::path(
    post,
    path = "/api/v1/tools/execute",
    request_body = ExecuteToolRequest,
    responses(
        (status = 200, description = "Tool executed successfully", body = ExecuteToolResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Tool not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn execute_tool(
    State(state): State<AppState>,
    Json(request): Json<ExecuteToolRequest>,
) -> ApiResult<Json<ExecuteToolResponse>> {
    // Validate request
    if request.tool_name.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Tool name cannot be empty".to_string(),
        ));
    }
    if request.tool_name.len() > 100 {
        return Err(ApiError::BadRequest(
            "Tool name too long (max 100 characters)".to_string(),
        ));
    }
    if !request
        .tool_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ApiError::BadRequest(
            "Tool name can only contain alphanumeric characters, underscores, and hyphens"
                .to_string(),
        ));
    }

    // Validate parameters
    if let Some(timeout) = request.timeout_seconds {
        if timeout > 300 {
            // Max 5 minutes
            return Err(ApiError::BadRequest(
                "Timeout too long (max 300 seconds)".to_string(),
            ));
        }
    }

    let tool_invoker = state
        .mcp_tool_invoker
        .as_ref()
        .ok_or_else(|| ApiError::Internal("MCP tool invoker not configured".to_string()))?;

    // Convert parameters to HashMap
    let parameters = if let serde_json::Value::Object(map) = request.parameters {
        // Validate parameter size
        for (key, value) in &map {
            if key.len() > 100 {
                return Err(ApiError::BadRequest(format!(
                    "Parameter name '{}' too long",
                    key
                )));
            }
            let value_str = serde_json::to_string(value)?;
            if value_str.len() > 10000 {
                return Err(ApiError::BadRequest(format!(
                    "Parameter '{}' value too large",
                    key
                )));
            }
        }
        map.into_iter().collect()
    } else {
        return Err(ApiError::BadRequest(
            "Parameters must be a JSON object".to_string(),
        ));
    };

    let start_time = std::time::Instant::now();

    // Execute the tool
    let result = tool_invoker.invoke_tool(&request.tool_name, parameters)?;

    let execution_time = start_time.elapsed().as_millis() as u64;

    let response = ExecuteToolResponse {
        result,
        execution_time_ms: execution_time,
        success: true,
    };

    Ok(Json(response))
}

/// List available MCP tools
#[utoipa::path(
    get,
    path = "/api/v1/tools",
    responses(
        (status = 200, description = "List of available tools", body = Vec<String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_tools(State(state): State<AppState>) -> ApiResult<Json<Vec<String>>> {
    // For now, return a static list. In a real implementation,
    // this would query the MCP servers for available tools.
    let tools = vec![
        "read_file".to_string(),
        "write_file".to_string(),
        "list_dir".to_string(),
        "search".to_string(),
        "run_command".to_string(),
    ];

    Ok(Json(tools))
}
