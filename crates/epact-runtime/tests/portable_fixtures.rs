use std::{fs, path::PathBuf};

use epact_compiler::{compile_program, verify_program_image};
use epact_protocol::{
    EpactEligibility, EpactOperationRequest, EpactProgram, EpactProgramImage, EpactRuntimeEvent,
    EpactRuntimeState,
};
use epact_runtime::{evaluate_epact_operation, replay_epact_events};
use serde::de::DeserializeOwned;

#[test]
fn committed_alpha_fixtures_recompile_replay_and_evaluate_exactly() {
    let source: EpactProgram = fixture("program.json");
    let image: EpactProgramImage = fixture("image.json");
    assert_eq!(compile_program(source).unwrap(), image);
    verify_program_image(&image).unwrap();

    let empty: Vec<EpactRuntimeEvent> = fixture("empty-events.json");
    let expected_initial: EpactRuntimeState = fixture("initial-state.json");
    assert_eq!(
        replay_epact_events(&image, &empty).unwrap(),
        expected_initial
    );

    let events: Vec<EpactRuntimeEvent> = fixture("events.json");
    let expected_state: EpactRuntimeState = fixture("state.json");
    assert_eq!(
        replay_epact_events(&image, &events).unwrap(),
        expected_state
    );

    let allowed_request: EpactOperationRequest = fixture("allowed-request.json");
    let allowed_result: EpactEligibility = fixture("allowed-result.json");
    assert!(allowed_result.allowed);
    assert_eq!(
        evaluate_epact_operation(&image, &expected_initial, &allowed_request).unwrap(),
        allowed_result
    );

    let denied_request: EpactOperationRequest = fixture("denied-request.json");
    let denied_result: EpactEligibility = fixture("denied-result.json");
    assert!(!denied_result.allowed);
    assert_eq!(
        evaluate_epact_operation(&image, &expected_initial, &denied_request).unwrap(),
        denied_result
    );
}

fn fixture<T: DeserializeOwned>(name: &str) -> T {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/alpha")
        .join(name);
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
