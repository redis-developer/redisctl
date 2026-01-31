//! Integration tests for Redis Enterprise MCP tools using mock server

use std::sync::Arc;

use redis_enterprise::testing::{
    ClusterFixture, DatabaseFixture, MockEnterpriseServer, NodeFixture, UserFixture,
};
use serde_json::json;
use tower_mcp::Tool;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

// Import the tools and state from the MCP crate
use redisctl_mcp::state::AppState;
use redisctl_mcp::tools::enterprise;

/// Helper to call a tool and get text result
async fn call_tool_text(tool: &Tool, input: serde_json::Value) -> String {
    let result = tool.call(input).await.expect("tool call succeeded");
    result
        .content
        .first()
        .and_then(|c| c.as_text())
        .unwrap_or_default()
        .to_string()
}

/// Helper to call a tool and get JSON result
async fn call_tool_json(tool: &Tool, input: serde_json::Value) -> serde_json::Value {
    let text = call_tool_text(tool, input).await;
    serde_json::from_str(&text).expect("valid JSON response")
}

// ============================================================================
// Cluster Tests
// ============================================================================

#[tokio::test]
async fn test_get_cluster() {
    let server = MockEnterpriseServer::start().await;

    let cluster = ClusterFixture::new("production-cluster")
        .nodes(vec![1, 2, 3])
        .build();

    server.mock_cluster_info(cluster).await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::get_cluster(state);

    let result = call_tool_json(&tool, json!({})).await;

    assert_eq!(result["name"], "production-cluster");
}

#[tokio::test]
async fn test_get_cluster_stats() {
    let server = MockEnterpriseServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/cluster/stats/last"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "avg_latency": 0.5,
            "total_req": 10000,
            "egress_bytes": 1024000,
            "ingress_bytes": 512000
        })))
        .mount(server.inner())
        .await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::get_cluster_stats(state);

    let result = call_tool_json(&tool, json!({})).await;

    assert!(result.get("avg_latency").is_some() || result.get("total_req").is_some());
}

// ============================================================================
// Database Tests
// ============================================================================

#[tokio::test]
async fn test_list_enterprise_databases() {
    let server = MockEnterpriseServer::start().await;

    let db1 = DatabaseFixture::new(1, "cache-primary")
        .memory_size(2 * 1024 * 1024 * 1024)
        .build();

    let db2 = DatabaseFixture::new(2, "sessions")
        .memory_size(1024 * 1024 * 1024)
        .build();

    server.mock_databases_list(vec![db1, db2]).await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::list_databases(state);

    let result = call_tool_text(&tool, json!({})).await;

    assert!(result.contains("cache-primary"));
    assert!(result.contains("sessions"));
    assert!(result.contains("2 database(s)"));
}

#[tokio::test]
async fn test_list_enterprise_databases_with_filter() {
    let server = MockEnterpriseServer::start().await;

    let db1 = DatabaseFixture::new(1, "cache-primary").build();
    let db2 = DatabaseFixture::new(2, "sessions").build();
    let db3 = DatabaseFixture::new(3, "cache-replica").build();

    server.mock_databases_list(vec![db1, db2, db3]).await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::list_databases(state);

    let result = call_tool_text(&tool, json!({"name_filter": "cache"})).await;

    assert!(result.contains("cache-primary"));
    assert!(result.contains("cache-replica"));
    assert!(!result.contains("sessions")); // Should be filtered out
    assert!(result.contains("2 database(s)"));
}

#[tokio::test]
async fn test_get_enterprise_database() {
    let server = MockEnterpriseServer::start().await;

    let database = DatabaseFixture::new(1, "cache-primary")
        .memory_size(2 * 1024 * 1024 * 1024)
        .build();

    server.mock_database_get(1, database).await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::get_database(state);

    let result = call_tool_json(&tool, json!({"uid": 1})).await;

    assert_eq!(result["uid"], 1);
    assert_eq!(result["name"], "cache-primary");
}

// ============================================================================
// Node Tests
// ============================================================================

#[tokio::test]
async fn test_list_nodes() {
    let server = MockEnterpriseServer::start().await;

    let node1 = NodeFixture::new(1, "10.0.0.1").cores(8).build();
    let node2 = NodeFixture::new(2, "10.0.0.2").cores(8).build();
    let node3 = NodeFixture::new(3, "10.0.0.3").cores(4).build();

    server.mock_nodes_list(vec![node1, node2, node3]).await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::list_nodes(state);

    let result = call_tool_text(&tool, json!({})).await;

    assert!(result.contains("10.0.0.1"));
    assert!(result.contains("10.0.0.2"));
    assert!(result.contains("10.0.0.3"));
    assert!(result.contains("3 node(s)"));
}

#[tokio::test]
async fn test_get_node() {
    let server = MockEnterpriseServer::start().await;

    let node = NodeFixture::new(1, "10.0.0.1").cores(8).build();

    server.mock_node_get(1, node).await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::get_node(state);

    let result = call_tool_json(&tool, json!({"uid": 1})).await;

    assert_eq!(result["uid"], 1);
    assert_eq!(result["addr"], "10.0.0.1");
}

// ============================================================================
// User Tests
// ============================================================================

#[tokio::test]
async fn test_list_enterprise_users() {
    let server = MockEnterpriseServer::start().await;

    let user1 = UserFixture::new(1, "admin@example.com")
        .name("Admin User")
        .build();

    let user2 = UserFixture::new(2, "dev@example.com")
        .name("Developer")
        .build();

    server.mock_users_list(vec![user1, user2]).await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::list_users(state);

    let result = call_tool_text(&tool, json!({})).await;

    assert!(result.contains("admin@example.com"));
    assert!(result.contains("dev@example.com"));
    assert!(result.contains("2 user(s)"));
}

#[tokio::test]
async fn test_get_enterprise_user() {
    let server = MockEnterpriseServer::start().await;

    let user = UserFixture::new(1, "admin@example.com")
        .name("Admin User")
        .build();

    server.mock_user_get(1, user).await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::get_user(state);

    let result = call_tool_json(&tool, json!({"uid": 1})).await;

    assert_eq!(result["uid"], 1);
    assert_eq!(result["email"], "admin@example.com");
}

// ============================================================================
// Alert and Stats Tests
// ============================================================================

#[tokio::test]
async fn test_list_alerts() {
    let server = MockEnterpriseServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "uid": "1",
                "name": "high_memory_usage",
                "severity": "warning",
                "enabled": true,
                "state": "on",
                "threshold": "70%",
                "cluster_alert": true
            }
        ])))
        .mount(server.inner())
        .await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::list_alerts(state);

    let result = call_tool_json(&tool, json!({})).await;

    assert!(result.is_array());
}

#[tokio::test]
async fn test_get_database_stats() {
    let server = MockEnterpriseServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/1/stats/last"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "avg_latency": 0.3,
            "total_req": 5000,
            "used_memory": 1024000
        })))
        .mount(server.inner())
        .await;

    let client = server.client();
    let state = Arc::new(AppState::with_enterprise_client(client));
    let tool = enterprise::get_database_stats(state);

    let result = call_tool_json(&tool, json!({"uid": 1})).await;

    assert!(result.get("avg_latency").is_some() || result.get("total_req").is_some());
}
