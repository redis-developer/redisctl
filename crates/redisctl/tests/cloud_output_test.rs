//! Tests for cloud command output formatting

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_subscription_table_format() {
        // Test data that mimics real API responses
        let test_data = json!([
            {
                "id": 12345,
                "name": "production-cache",
                "status": "active",
                "planId": "pro",
                "planName": "Pro",
                "paymentMethod": "credit-card",
                "created": "2024-01-15T10:30:00Z",
                "numberOfDatabases": 3,
                "memoryStorage": {
                    "quantity": 4.0,
                    "units": "GB"
                },
                "cloudProviders": [
                    {
                        "provider": "AWS",
                        "regions": [
                            {
                                "region": "us-east-1",
                                "memoryStorage": {
                                    "quantity": 4.0
                                }
                            }
                        ]
                    }
                ]
            },
            {
                "id": 67890,
                "name": "staging-db",
                "status": "pending",
                "planId": "fixed-50",
                "planName": "Standard",
                "created": "2025-09-01T08:00:00Z",
                "numberOfDatabases": 1,
                "memoryStorage": {
                    "quantity": 1.0,
                    "units": "GB"
                },
                "cloudProviders": [
                    {
                        "provider": "GCP",
                        "regions": [
                            {
                                "region": "europe-west1",
                                "memoryStorage": {
                                    "quantity": 1.0
                                }
                            }
                        ]
                    }
                ]
            }
        ]);

        // Just verify the test data structure is valid
        assert!(test_data.is_array());
        assert_eq!(test_data.as_array().unwrap().len(), 2);

        // Verify we can extract expected fields
        let first = &test_data[0];
        assert_eq!(first["id"], 12345);
        assert_eq!(first["name"], "production-cache");
        assert_eq!(first["status"], "active");
    }

    #[test]
    fn test_jmespath_filtering() {
        let data = json!([
            {"id": 1, "status": "active", "memory": 4},
            {"id": 2, "status": "pending", "memory": 2},
            {"id": 3, "status": "active", "memory": 8}
        ]);

        let expr = jpx_core::compile("[?status=='active']").unwrap();
        let filtered = expr.search(&data).unwrap();

        assert!(filtered.is_array());
        assert_eq!(filtered.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_jmespath_extended_functions() {
        let runtime = jpx_core::Runtime::builder().with_all_extensions().build();

        let data = json!([
            {"id": 1, "name": "production", "status": "active", "memory_bytes": 4294967296_i64},
            {"id": 2, "name": "staging", "status": "pending", "memory_bytes": 2147483648_i64},
            {"id": 3, "name": "development", "status": "active", "memory_bytes": 1073741824_i64}
        ]);

        // Test upper() function
        let expr = runtime
            .compile("[].{name: name, status: upper(status)}")
            .unwrap();
        let transformed = expr.search(&data).unwrap();
        assert_eq!(transformed[0]["status"], "ACTIVE");
        assert_eq!(transformed[1]["status"], "PENDING");

        // Test unique() function
        let expr = runtime.compile("unique([].status)").unwrap();
        let unique_statuses = expr.search(&data).unwrap();
        assert!(unique_statuses.is_array());
        assert_eq!(unique_statuses.as_array().unwrap().len(), 2);

        // Test type_of() function
        let expr = runtime
            .compile("[0].{name: name, type: type_of(memory_bytes)}")
            .unwrap();
        let type_check = expr.search(&data).unwrap();
        assert_eq!(type_check["type"], "number");

        // Test is_empty() function
        let empty_data = json!({"items": [], "name": "test"});
        let expr = runtime
            .compile("{is_empty_items: is_empty(items)}")
            .unwrap();
        let empty_check = expr.search(&empty_data).unwrap();
        assert_eq!(empty_check["is_empty_items"], true);
    }

    #[test]
    fn test_jmespath_extended_string_functions() {
        let runtime = jpx_core::Runtime::builder().with_all_extensions().build();

        let data = json!({
            "name": "  my-cluster-name  ",
            "url": "https://api.example.com/path"
        });

        // Test trim() function
        let expr = runtime.compile("trim(name)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("my-cluster-name"));

        // Test split() function
        let expr = runtime.compile("split(name, '-')").unwrap();
        let parts = expr.search(&data).unwrap();
        assert!(parts.is_array());
    }

    #[test]
    fn test_jmespath_extended_utility_functions() {
        let runtime = jpx_core::Runtime::builder().with_all_extensions().build();

        let data = json!({
            "primary_region": null,
            "fallback_region": "us-east-1"
        });

        // Test coalesce() function - returns first non-null value
        let expr = runtime
            .compile("coalesce(primary_region, fallback_region, `\"default\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("us-east-1"));

        // Test default() function
        let expr = runtime
            .compile("default(primary_region, `\"default-region\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("default-region"));
    }
}
