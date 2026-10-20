//! YAML represents the same JSON-compatible data, streams and exits as JSON.
//! Fixtures are synthetic; Cloud calls, when enabled, target only a loopback mock.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{Value, json};
use std::time::Duration;
use tempfile::TempDir;

fn command(directory: &TempDir) -> Command {
    let mut command = Command::cargo_bin("redisctl").unwrap();
    command
        .env_clear()
        .env("HOME", directory.path())
        .env("XDG_CONFIG_HOME", directory.path())
        .arg("--config-file")
        .arg(directory.path().join("config.toml"))
        .timeout(Duration::from_secs(5));
    command
}

#[test]
fn yaml_preserves_json_scalars_containers_and_string_keys() {
    let mut cases = vec![
        Value::Null,
        json!([true, false]),
        json!([
            i64::MIN,
            i64::MAX,
            u64::MAX,
            0,
            -7,
            1.5,
            -0.0,
            1.0,
            1e30,
            1e-30
        ]),
        json!({"array": [], "map": {}, "string": ""}),
        json!({"items":[{"id":1,"active":true,"name":"世界"},null],"defaults":{"cloud":null,"database":"demo"}}),
        json!({"error":{"code":"invalid_input","exit_code":5,"message":"Invalid input: expected a number: got 'true'","tips":["Try --help", "Use a valid profile."]}}),
    ];
    for text in [
        "",
        "null",
        "Null",
        "NULL",
        "~",
        "true",
        "False",
        "yes",
        "no",
        "on",
        "off",
        "y",
        "N",
        "0",
        "01",
        "-0",
        "1.0",
        "1e3",
        "0x10",
        "12:34:56",
        "2026-10-19",
        ".nan",
        ".inf",
        "-.Inf",
        "a: b",
        "a # b",
        "- item",
        "? key",
        "[x]",
        "{a: b}",
        "&anchor",
        "*alias",
        "!tag",
        "---",
        "...",
        " leading",
        "trailing ",
        "a\tb",
        "a\r\nb",
        "a\0b",
        "a\u{7}b",
        "a\u{85}b",
        "a\u{2028}b",
        "a\u{2029}b",
        "é雪🔑",
        "'single' and \"double\"",
        "C:\\fixture\\config.toml",
        "${HOME} !include ./file.yaml",
        "a\nb",
        "a\nb\n",
        "a\nb\n\n",
        "This is a long synthetic string with enough spaces to exceed eighty characters and exercise wrapping without contacting any service.",
    ] {
        cases.push(json!(text));
        cases.push(json!({text: text}));
    }
    assert_eq!(cases.len(), 110);
    for value in cases {
        let encoded = serde_saphyr::to_string(&value).unwrap();
        let decoded: Value = serde_saphyr::from_str(&encoded).unwrap();
        assert_eq!(decoded, value, "YAML: {encoded:?}");
    }
    let encoded = serde_saphyr::to_string(&json!(-0.0)).unwrap();
    let decoded: Value = serde_saphyr::from_str(&encoded).unwrap();
    assert!(decoded.as_f64().unwrap().is_sign_negative());
}

#[test]
fn yaml_selected_formatting_is_explicit() {
    // Intentional migration snapshots, not a promise of byte-identical legacy output.
    for (value, expected) in [
        (json!("yes"), "\"yes\"\n"),
        (json!("a\u{2028}b"), "\"a\\Lb\"\n"),
        (json!("a\nb\n"), "|\n  a\n  b\n"),
        (
            json!([i64::MIN, u64::MAX, -0.0, 1e30]),
            "- -9223372036854775808\n- 18446744073709551615\n- -0.0\n- 1.0e+30\n",
        ),
        (
            json!({"empty":[],"items":[{"id":1}]}),
            "empty: []\nitems:\n- id: 1\n",
        ),
    ] {
        assert_eq!(serde_saphyr::to_string(&value).unwrap(), expected);
        assert_eq!(serde_saphyr::from_str::<Value>(expected).unwrap(), value);
    }
}

#[test]
fn yaml_version_success_preserves_stdout_data_and_newlines() {
    let directory = TempDir::new().unwrap();
    let output = command(&directory)
        .args(["version", "--output", "yaml"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
    // Other workspace crates may enable serde_json/preserve_order. Read JSON from
    // this same binary so this checks stream/newline parity, not a map-order assumption.
    let json_output = command(&directory)
        .args(["version", "--output", "json"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
    let expected: Value = serde_json::from_slice(&json_output.get_output().stdout).unwrap();
    assert_eq!(expected["name"], "redisctl");
    assert_eq!(expected["version"], env!("CARGO_PKG_VERSION"));
    let encoded = serde_saphyr::to_string(&expected).unwrap();
    assert_eq!(
        output.get_output().stdout,
        format!("{encoded}\n").as_bytes()
    );
    assert_eq!(
        serde_saphyr::from_slice::<Value>(&output.get_output().stdout).unwrap(),
        expected
    );
    assert!(!directory.path().join("config.toml").exists());
}

#[cfg(feature = "cloud")]
#[test]
fn yaml_cancellation_preserves_the_complete_json_error_and_stderr() {
    let directory = TempDir::new().unwrap();
    let json_output = command(&directory)
        .args(["cloud", "user", "delete", "42", "--output", "json"])
        .write_stdin("n\n")
        .assert()
        .code(12)
        .stdout(predicate::str::is_empty());
    let expected: Value = serde_json::from_slice(&json_output.get_output().stderr).unwrap();
    let yaml_output = command(&directory)
        .args(["cloud", "user", "delete", "42", "--output", "yaml"])
        .write_stdin("n\n")
        .assert()
        .code(12)
        .stdout(predicate::str::is_empty());
    let actual: Value = serde_saphyr::from_slice(&yaml_output.get_output().stderr).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual["error"]["code"], "cancelled");
    assert_eq!(actual["error"]["exit_code"], 12);
    assert_eq!(
        yaml_output.get_output().stderr,
        serde_saphyr::to_string(&expected).unwrap().as_bytes()
    );
    assert!(!directory.path().join("config.toml").exists());
}

#[cfg(feature = "cloud")]
#[test]
fn yaml_agent_workflow_precondition_preserves_stdout_error() {
    // Explicit missing profile fails before client construction, API access or file writes.
    let directory = TempDir::new().unwrap();
    let mut outputs = Vec::new();
    for format in ["json", "yaml"] {
        let output = command(&directory)
            .current_dir(directory.path())
            .args([
                "--profile",
                "missing-fixture",
                "cloud",
                "workflow",
                "database-credentials",
                "--subscription-id",
                "1",
                "--database-id",
                "2",
                "--output",
                format,
            ])
            .arg("--output-credentials")
            .arg(directory.path().join("fixture.env"))
            .assert()
            .code(2)
            .stderr(predicate::str::is_empty());
        outputs.push(output.get_output().stdout.clone());
    }
    let expected: Value = serde_json::from_slice(&outputs[0]).unwrap();
    let actual: Value = serde_saphyr::from_slice(&outputs[1]).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual["error"]["code"], "not_authenticated");
    assert_eq!(
        outputs[1],
        format!("{}\n", serde_saphyr::to_string(&expected).unwrap()).as_bytes()
    );
    assert!(!directory.path().join("config.toml").exists());
    assert!(!directory.path().join("fixture.env").exists());
}

#[cfg(feature = "cloud")]
#[tokio::test]
async fn yaml_mocked_success_and_query_roots_preserve_json_values() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let directory = TempDir::new().unwrap();
    let server = MockServer::start().await;
    let config = format!(
        "default_cloud = \"fixture\"\n[profiles.fixture]\ndeployment_type = \"cloud\"\napi_key = \"synthetic-key\"\napi_secret = \"synthetic-secret\"\napi_url = \"{}\"\n",
        server.uri()
    );
    std::fs::write(directory.path().join("config.toml"), &config).unwrap();
    let payload = json!({"name":"yes", "nested":{"snow":"雪", "empty":[]}, "items":[{"id":1},{"id":2}], "enabled":true});
    let cases = [
        ("", payload.clone()),
        ("name", json!("yes")),
        ("nested", payload["nested"].clone()),
        ("items[].id", json!([1, 2])),
        ("nested.empty", json!([])),
        ("enabled", json!(true)),
        ("missing", Value::Null),
    ];
    Mock::given(method("GET"))
        .and(path("/fixture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
        .expect(cases.len() as u64)
        .mount(&server)
        .await;
    for (query, expected) in cases {
        let mut command = command(&directory);
        command.args(["api", "cloud", "get", "/fixture", "--output", "yaml"]);
        if !query.is_empty() {
            command.args(["--query", query]);
        }
        let output = command
            .assert()
            .success()
            .stderr(predicate::str::is_empty());
        let actual: Value = serde_saphyr::from_slice(&output.get_output().stdout).unwrap();
        assert_eq!(actual, expected, "query={query}");
        let expected_bytes = format!("{}\n", serde_saphyr::to_string(&expected).unwrap());
        assert_eq!(
            output.get_output().stdout,
            expected_bytes.as_bytes(),
            "query={query}"
        );
    }
    assert_eq!(
        std::fs::read_to_string(directory.path().join("config.toml")).unwrap(),
        config
    );
}
