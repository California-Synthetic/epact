use std::{fs, path::PathBuf};

use epact_protocol::EpactProgram;
use serde_json::{json, Value};

#[test]
fn unknown_policy_fields_fail_closed_at_every_depth() {
    let source = fs::read(fixture_path("program.json")).unwrap();
    let canonical: Value = serde_json::from_slice(&source).unwrap();
    assert!(serde_json::from_value::<EpactProgram>(canonical.clone()).is_ok());

    let mut unknown_root = canonical.clone();
    unknown_root["resourceCeilings"] = json!({});
    assert!(serde_json::from_value::<EpactProgram>(unknown_root).is_err());

    let mut unknown_obligation = canonical.clone();
    unknown_obligation["obligations"][0]["maximumRetries"] = json!(1);
    assert!(serde_json::from_value::<EpactProgram>(unknown_obligation).is_err());

    let mut unknown_discharge = canonical;
    unknown_discharge["obligations"][0]["discharge"]["fallbackCapabilityId"] =
        json!("capability:other");
    assert!(serde_json::from_value::<EpactProgram>(unknown_discharge).is_err());
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/alpha")
        .join(name)
}
