use std::{error::Error, fs, path::PathBuf};

use epact_compiler::{compile_program, EPACT_COMPILER_VERSION};
use epact_protocol::{
    canonical_epact_json_bytes, EffectClass, EpactAmendmentPolicy, EpactAuthorityGrant,
    EpactAuthorityScope, EpactCapabilityRequirement, EpactDischarge, EpactObjectDeclaration,
    EpactObligation, EpactOperationRequest, EpactPrincipal, EpactProgram, EpactResourceEnvelope,
    EpactRuntimeEvent, EpactRuntimeEventKind, EpactTerminalRule, KernelOperation, PrincipalKind,
    ProgramLifecycle, ReversibilityClass, ReversibilityPolicy, EPACT_PROGRAM_CONTRACT,
};
use epact_runtime::{evaluate_epact_operation, initial_epact_state, replay_epact_events};
use serde_json::json;

const RECEIPT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/alpha");
    fs::create_dir_all(&directory)?;

    let program = fixture_program();
    let image = compile_program(program.clone())?;
    let initial_state = initial_epact_state(&image)?;
    let events = fixture_events(&image.image_sha256)?;
    let state = replay_epact_events(&image, &events)?;
    let allowed_request = request(vec![EffectClass::ReadOnly]);
    let allowed = evaluate_epact_operation(&image, &initial_state, &allowed_request)?;
    let denied_request = request(vec![EffectClass::ExternalWrite]);
    let denied = evaluate_epact_operation(&image, &initial_state, &denied_request)?;

    write(&directory, "program.json", &program)?;
    write(&directory, "image.json", &image)?;
    write(
        &directory,
        "empty-events.json",
        &Vec::<EpactRuntimeEvent>::new(),
    )?;
    write(&directory, "initial-state.json", &initial_state)?;
    write(&directory, "events.json", &events)?;
    write(&directory, "state.json", &state)?;
    write(&directory, "allowed-request.json", &allowed_request)?;
    write(&directory, "allowed-result.json", &allowed)?;
    write(&directory, "denied-request.json", &denied_request)?;
    write(&directory, "denied-result.json", &denied)?;
    write(
        &directory,
        "manifest.json",
        &json!({
            "contract": "epact.conformance-manifest/0.1-alpha",
            "compilerVersion": EPACT_COMPILER_VERSION,
            "programSha256": image.program_sha256,
            "imageSha256": image.image_sha256,
            "eventHeadSha256": state.event_head_sha256,
            "files": [
                "program.json",
                "image.json",
                "empty-events.json",
                "initial-state.json",
                "events.json",
                "state.json",
                "allowed-request.json",
                "allowed-result.json",
                "denied-request.json",
                "denied-result.json"
            ]
        }),
    )?;
    Ok(())
}

fn write(
    directory: &std::path::Path,
    name: &str,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn Error>> {
    let mut bytes = canonical_epact_json_bytes(value)?;
    bytes.push(b'\n');
    fs::write(directory.join(name), bytes)?;
    Ok(())
}

fn fixture_program() -> EpactProgram {
    let resources = EpactResourceEnvelope {
        maximum_elapsed_seconds: 120,
        maximum_model_calls: 1,
        maximum_tool_calls: 1,
        maximum_cpu_cores: 2.0,
        maximum_ram_gb: 4.0,
        ..EpactResourceEnvelope::default()
    };
    EpactProgram {
        contract: EPACT_PROGRAM_CONTRACT.to_owned(),
        id: "program:alpha-conformance".to_owned(),
        version: "1".to_owned(),
        title: "Epact alpha conformance".to_owned(),
        lifecycle: ProgramLifecycle::Frozen,
        created_by: "principal:operator".to_owned(),
        predecessor: None,
        imports: vec![],
        principals: vec![
            EpactPrincipal {
                id: "principal:operator".to_owned(),
                kind: PrincipalKind::Human,
                display_name: "Operator".to_owned(),
            },
            EpactPrincipal {
                id: "principal:agent".to_owned(),
                kind: PrincipalKind::Agent,
                display_name: "Workbench agent".to_owned(),
            },
        ],
        objects: vec![EpactObjectDeclaration {
            id: "object:result".to_owned(),
            type_name: "example.analysis/1".to_owned(),
            schema_sha256: None,
            data_classes: vec![],
        }],
        capabilities: vec![EpactCapabilityRequirement {
            id: "capability:analyze".to_owned(),
            capability_type: "deterministic_analysis".to_owned(),
            contract: "example.analysis/1".to_owned(),
            required_effects: vec![EffectClass::ReadOnly],
            required_data_classes: vec![],
            placement: None,
        }],
        authorities: vec![
            EpactAuthorityGrant {
                id: "authority:agent:analyze".to_owned(),
                principal_id: "principal:agent".to_owned(),
                operations: vec![
                    KernelOperation::Propose,
                    KernelOperation::Reserve,
                    KernelOperation::Dispatch,
                ],
                scope: EpactAuthorityScope {
                    whole_program: false,
                    obligation_ids: vec!["analyze".to_owned()],
                    capability_ids: vec!["capability:analyze".to_owned()],
                },
                maximum_cost_usd: 0.0,
                valid_after: None,
                valid_before: None,
            },
            EpactAuthorityGrant {
                id: "authority:operator".to_owned(),
                principal_id: "principal:operator".to_owned(),
                operations: vec![
                    KernelOperation::Freeze,
                    KernelOperation::Authorize,
                    KernelOperation::Amend,
                ],
                scope: EpactAuthorityScope {
                    whole_program: true,
                    obligation_ids: vec![],
                    capability_ids: vec![],
                },
                maximum_cost_usd: 0.0,
                valid_after: None,
                valid_before: None,
            },
        ],
        resources: resources.clone(),
        obligations: vec![EpactObligation {
            id: "analyze".to_owned(),
            label: "Analyze".to_owned(),
            description: "Produce one bounded deterministic result.".to_owned(),
            dependency_ids: vec![],
            gate_ids: vec![],
            discharge: EpactDischarge::Capability {
                capability_id: "capability:analyze".to_owned(),
            },
            output_object_ids: vec!["object:result".to_owned()],
            effects: vec![EffectClass::ReadOnly],
            resources,
            reversibility: ReversibilityPolicy {
                class: ReversibilityClass::ReadOnly,
                reversal_action: None,
                limitations: vec![],
            },
            retry_limit: 1,
            terminal_receipt_contract: "example.analysis-receipt/1".to_owned(),
        }],
        gates: vec![],
        evidence_rules: vec![],
        amendment_policy: EpactAmendmentPolicy {
            authorized_principal_ids: vec!["principal:operator".to_owned()],
            rationale_required: true,
            effective_causal_head_required: true,
            preserve_prior_interpretation: true,
        },
        terminal: EpactTerminalRule {
            required_obligation_ids: vec!["analyze".to_owned()],
            required_object_ids: vec!["object:result".to_owned()],
            required_receipt_contracts: vec!["example.analysis-receipt/1".to_owned()],
        },
    }
}

fn fixture_events(image_sha256: &str) -> Result<Vec<EpactRuntimeEvent>, Box<dyn Error>> {
    let first = EpactRuntimeEvent::build(
        "event:0".to_owned(),
        image_sha256.to_owned(),
        0,
        "principal:agent".to_owned(),
        "fixture:object".to_owned(),
        EpactRuntimeEventKind::ObjectRecorded {
            object_id: "object:result".to_owned(),
        },
        Some(RECEIPT.to_owned()),
        None,
        "2026-09-03T00:00:00Z".to_owned(),
    )?;
    let second = EpactRuntimeEvent::build(
        "event:1".to_owned(),
        image_sha256.to_owned(),
        1,
        "principal:agent".to_owned(),
        "fixture:terminal".to_owned(),
        EpactRuntimeEventKind::ObligationSatisfied {
            obligation_id: "analyze".to_owned(),
            receipt_contract: "example.analysis-receipt/1".to_owned(),
        },
        Some(RECEIPT.to_owned()),
        Some(first.event_sha256.clone()),
        "2026-09-03T00:00:01Z".to_owned(),
    )?;
    Ok(vec![first, second])
}

fn request(effects: Vec<EffectClass>) -> EpactOperationRequest {
    EpactOperationRequest {
        principal_id: "principal:agent".to_owned(),
        operation: KernelOperation::Dispatch,
        requested_at: "2026-09-03T00:00:02Z".to_owned(),
        obligation_id: Some("analyze".to_owned()),
        capability_id: Some("capability:analyze".to_owned()),
        effects,
        resources: EpactResourceEnvelope {
            maximum_elapsed_seconds: 60,
            maximum_tool_calls: 1,
            maximum_cpu_cores: 1.0,
            maximum_ram_gb: 2.0,
            ..EpactResourceEnvelope::default()
        },
        placement: None,
    }
}
