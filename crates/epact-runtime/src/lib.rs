use std::collections::BTreeSet;

use epact_compiler::{require_activatable, verify_program_image};
use epact_protocol::{
    validate_epact_timestamp, CompiledAuthority, EpactAcceptedReview, EpactDischarge,
    EpactEligibility, EpactEligibilityBlocker, EpactObligation, EpactObligationProjection,
    EpactObligationState, EpactOperationRequest, EpactPredicate, EpactProgramImage,
    EpactResourceEnvelope, EpactRuntimeEvent, EpactRuntimeEventKind, EpactRuntimeState,
    KernelOperation,
};
use thiserror::Error;

/// Construct the projection produced by replaying an empty history under one compiled image.
pub fn initial_epact_state(
    image: &EpactProgramImage,
) -> Result<EpactRuntimeState, EpactRuntimeError> {
    verify_program_image(image)
        .map_err(|error| EpactRuntimeError::InvalidImage(error.to_string()))?;
    Ok(EpactRuntimeState {
        program_image_sha256: image.image_sha256.clone(),
        next_sequence: 0,
        event_head_sha256: None,
        obligations: image
            .obligation_order
            .iter()
            .map(|obligation_id| EpactObligationProjection {
                obligation_id: obligation_id.clone(),
                state: EpactObligationState::Pending,
                terminal_event_sha256: None,
            })
            .collect(),
        present_object_ids: Vec::new(),
        satisfied_evidence_rule_ids: Vec::new(),
        accepted_reviews: Vec::new(),
    })
}

/// Rebuild all authoritative Epact projections from the compiled image and its accepted facts.
///
/// Event hashes prove integrity and order. Receipt contents remain kernel-owned evidence; replay
/// never upgrades a digest into scientific truth.
pub fn replay_epact_events(
    image: &EpactProgramImage,
    events: &[EpactRuntimeEvent],
) -> Result<EpactRuntimeState, EpactRuntimeError> {
    let mut state = initial_epact_state(image)?;
    let mut event_ids = BTreeSet::new();
    let mut idempotency_keys = BTreeSet::new();

    for event in events {
        event
            .validate()
            .map_err(|error| EpactRuntimeError::InvalidEvent(error.to_string()))?;
        if event.program_image_sha256 != image.image_sha256 {
            return Err(EpactRuntimeError::ImageBindingMismatch);
        }
        if event.sequence != state.next_sequence {
            return Err(EpactRuntimeError::UnexpectedSequence {
                expected: state.next_sequence,
                actual: event.sequence,
            });
        }
        if event.previous_event_sha256 != state.event_head_sha256 {
            return Err(EpactRuntimeError::BrokenEventChain(event.id.clone()));
        }
        if !event_ids.insert(event.id.clone()) {
            return Err(EpactRuntimeError::DuplicateEventId(event.id.clone()));
        }
        if !idempotency_keys.insert(event.idempotency_key.clone()) {
            return Err(EpactRuntimeError::DuplicateIdempotencyKey(
                event.idempotency_key.clone(),
            ));
        }
        if !image
            .program
            .principals
            .iter()
            .any(|principal| principal.id == event.actor)
        {
            return Err(EpactRuntimeError::UnknownPrincipal(event.actor.clone()));
        }
        verify_epact_event_authority(image, &state, event)?;

        apply_event(image, &mut state, event)?;
        state.next_sequence += 1;
        state.event_head_sha256 = Some(event.event_sha256.clone());
    }
    Ok(state)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct EventAuthorityPath<'a> {
    operation: KernelOperation,
    obligation_id: Option<&'a str>,
    capability_id: Option<&'a str>,
}

/// Verify that the actor recording a runtime fact owns at least one authority path capable of
/// producing that fact under the current projection.
///
/// Integrity alone is insufficient: without this check, a declared but unrelated principal could
/// append a validly hashed event. Kernels may call this before persistence; replay calls it again so
/// independent verifiers reach the same fail-closed result.
pub fn verify_epact_event_authority(
    image: &EpactProgramImage,
    state: &EpactRuntimeState,
    event: &EpactRuntimeEvent,
) -> Result<(), EpactRuntimeError> {
    let paths = event_authority_paths(image, state, &event.kind)?;
    if paths.is_empty() {
        return Err(EpactRuntimeError::EventAuthorityPathUnavailable(
            event.id.clone(),
        ));
    }
    let allowed = paths.iter().any(|path| {
        image.authorities.iter().any(|authority| {
            authority.principal_id == event.actor
                && authority.operation == path.operation
                && (authority.whole_program
                    || path.obligation_id.is_some_and(|id| {
                        authority
                            .obligation_ids
                            .iter()
                            .any(|candidate| candidate == id)
                    })
                    || path.capability_id.is_some_and(|id| {
                        authority
                            .capability_ids
                            .iter()
                            .any(|candidate| candidate == id)
                    }))
                && authority
                    .valid_after
                    .as_ref()
                    .is_none_or(|after| &event.created_at >= after)
                && authority
                    .valid_before
                    .as_ref()
                    .is_none_or(|before| &event.created_at < before)
        })
    });
    if !allowed {
        return Err(EpactRuntimeError::EventAuthorityDenied {
            event_id: event.id.clone(),
            actor: event.actor.clone(),
        });
    }
    Ok(())
}

fn event_authority_paths<'a>(
    image: &'a EpactProgramImage,
    state: &EpactRuntimeState,
    kind: &'a EpactRuntimeEventKind,
) -> Result<Vec<EventAuthorityPath<'a>>, EpactRuntimeError> {
    let mut paths = match kind {
        EpactRuntimeEventKind::ObjectRecorded { object_id } => {
            object_authority_paths(image, object_id)
        }
        EpactRuntimeEventKind::EvidenceAccepted {
            evidence_rule_id, ..
        } => image
            .program
            .obligations
            .iter()
            .filter(|obligation| {
                discharge_uses_evidence_rule(&obligation.discharge, evidence_rule_id)
            })
            .map(|obligation| EventAuthorityPath {
                operation: KernelOperation::Evaluate,
                obligation_id: Some(obligation.id.as_str()),
                capability_id: None,
            })
            .collect(),
        EpactRuntimeEventKind::ReviewAccepted {
            obligation_id,
            review_object_id,
            ..
        } => {
            let obligation = find_obligation(image, obligation_id)?;
            review_authority_path(&obligation.discharge, review_object_id, obligation_id)
                .into_iter()
                .collect()
        }
        EpactRuntimeEventKind::ObligationSatisfied { obligation_id, .. } => {
            let obligation = find_obligation(image, obligation_id)?;
            satisfied_discharge_authority_paths(state, obligation, &obligation.discharge)
        }
        EpactRuntimeEventKind::ObligationFailed { obligation_id, .. }
        | EpactRuntimeEventKind::ObligationCancelled { obligation_id, .. } => {
            find_obligation(image, obligation_id)?;
            vec![EventAuthorityPath {
                operation: KernelOperation::Authorize,
                obligation_id: Some(obligation_id.as_str()),
                capability_id: None,
            }]
        }
    };
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn object_authority_paths<'a>(
    image: &'a EpactProgramImage,
    object_id: &str,
) -> Vec<EventAuthorityPath<'a>> {
    let mut paths = Vec::new();
    for obligation in &image.program.obligations {
        if obligation
            .output_object_ids
            .iter()
            .any(|candidate| candidate == object_id)
        {
            paths.extend(declared_discharge_authority_paths(
                obligation,
                &obligation.discharge,
            ));
        }
        paths.extend(discharge_object_authority_paths(
            image,
            obligation,
            &obligation.discharge,
            object_id,
        ));
    }
    paths
}

fn declared_discharge_authority_paths<'a>(
    obligation: &'a EpactObligation,
    discharge: &'a EpactDischarge,
) -> Vec<EventAuthorityPath<'a>> {
    match discharge {
        EpactDischarge::AnyOf { alternatives } => alternatives
            .iter()
            .flat_map(|alternative| declared_discharge_authority_paths(obligation, alternative))
            .collect(),
        EpactDischarge::Capability { capability_id } => vec![EventAuthorityPath {
            operation: KernelOperation::Dispatch,
            obligation_id: Some(obligation.id.as_str()),
            capability_id: Some(capability_id.as_str()),
        }],
        EpactDischarge::Decision { .. } => vec![EventAuthorityPath {
            operation: KernelOperation::Decide,
            obligation_id: Some(obligation.id.as_str()),
            capability_id: None,
        }],
        EpactDischarge::Evidence { .. } => vec![EventAuthorityPath {
            operation: KernelOperation::Evaluate,
            obligation_id: Some(obligation.id.as_str()),
            capability_id: None,
        }],
        EpactDischarge::Review { capability_id, .. } => vec![EventAuthorityPath {
            operation: KernelOperation::Evaluate,
            obligation_id: Some(obligation.id.as_str()),
            capability_id: Some(capability_id.as_str()),
        }],
        EpactDischarge::Publication { .. } => vec![EventAuthorityPath {
            operation: KernelOperation::Publish,
            obligation_id: Some(obligation.id.as_str()),
            capability_id: None,
        }],
    }
}

fn discharge_object_authority_paths<'a>(
    image: &'a EpactProgramImage,
    obligation: &'a EpactObligation,
    discharge: &'a EpactDischarge,
    object_id: &str,
) -> Vec<EventAuthorityPath<'a>> {
    match discharge {
        EpactDischarge::AnyOf { alternatives } => alternatives
            .iter()
            .flat_map(|alternative| {
                discharge_object_authority_paths(image, obligation, alternative, object_id)
            })
            .collect(),
        EpactDischarge::Decision { decision_object_id } if decision_object_id == object_id => {
            declared_discharge_authority_paths(obligation, discharge)
        }
        EpactDischarge::Evidence { evidence_rule_ids }
            if evidence_rule_ids.iter().any(|rule_id| {
                image.program.evidence_rules.iter().any(|rule| {
                    rule.id == *rule_id
                        && (rule.claim_object_id == object_id
                            || rule
                                .evidence_object_ids
                                .iter()
                                .any(|candidate| candidate == object_id))
                })
            }) =>
        {
            declared_discharge_authority_paths(obligation, discharge)
        }
        EpactDischarge::Review {
            review_object_id, ..
        } if review_object_id == object_id => {
            declared_discharge_authority_paths(obligation, discharge)
        }
        EpactDischarge::Publication {
            artifact_object_ids,
        } if artifact_object_ids
            .iter()
            .any(|candidate| candidate == object_id) =>
        {
            declared_discharge_authority_paths(obligation, discharge)
        }
        _ => Vec::new(),
    }
}

fn discharge_uses_evidence_rule(discharge: &EpactDischarge, evidence_rule_id: &str) -> bool {
    match discharge {
        EpactDischarge::AnyOf { alternatives } => alternatives
            .iter()
            .any(|alternative| discharge_uses_evidence_rule(alternative, evidence_rule_id)),
        EpactDischarge::Evidence { evidence_rule_ids } => evidence_rule_ids
            .iter()
            .any(|candidate| candidate == evidence_rule_id),
        _ => false,
    }
}

fn review_authority_path<'a>(
    discharge: &'a EpactDischarge,
    review_object_id: &str,
    obligation_id: &'a str,
) -> Option<EventAuthorityPath<'a>> {
    match discharge {
        EpactDischarge::AnyOf { alternatives } => alternatives.iter().find_map(|alternative| {
            review_authority_path(alternative, review_object_id, obligation_id)
        }),
        EpactDischarge::Review {
            capability_id,
            review_object_id: expected,
            ..
        } if expected == review_object_id => Some(EventAuthorityPath {
            operation: KernelOperation::Evaluate,
            obligation_id: Some(obligation_id),
            capability_id: Some(capability_id.as_str()),
        }),
        _ => None,
    }
}

fn satisfied_discharge_authority_paths<'a>(
    state: &EpactRuntimeState,
    obligation: &'a EpactObligation,
    discharge: &'a EpactDischarge,
) -> Vec<EventAuthorityPath<'a>> {
    match discharge {
        EpactDischarge::AnyOf { alternatives } => alternatives
            .iter()
            .flat_map(|alternative| {
                satisfied_discharge_authority_paths(state, obligation, alternative)
            })
            .collect(),
        EpactDischarge::Capability { .. } => {
            declared_discharge_authority_paths(obligation, discharge)
        }
        EpactDischarge::Decision { decision_object_id }
            if state
                .present_object_ids
                .binary_search(decision_object_id)
                .is_ok() =>
        {
            declared_discharge_authority_paths(obligation, discharge)
        }
        EpactDischarge::Evidence { evidence_rule_ids }
            if evidence_rule_ids.iter().all(|rule_id| {
                state
                    .satisfied_evidence_rule_ids
                    .binary_search(rule_id)
                    .is_ok()
            }) =>
        {
            declared_discharge_authority_paths(obligation, discharge)
        }
        EpactDischarge::Review {
            review_object_id, ..
        } if state
            .present_object_ids
            .binary_search(review_object_id)
            .is_ok()
            && state.accepted_reviews.iter().any(|review| {
                review.obligation_id == obligation.id
                    && review.review_object_id == *review_object_id
            }) =>
        {
            declared_discharge_authority_paths(obligation, discharge)
        }
        EpactDischarge::Publication {
            artifact_object_ids,
        } if artifact_object_ids
            .iter()
            .all(|object_id| state.present_object_ids.binary_search(object_id).is_ok()) =>
        {
            declared_discharge_authority_paths(obligation, discharge)
        }
        _ => Vec::new(),
    }
}

/// Decide whether a requested transition fits the frozen program and current projection.
///
/// This function is pure and provider-neutral. The kernel remains responsible for identity,
/// persistence, clocks, reservations, and effect execution.
pub fn evaluate_epact_operation(
    image: &EpactProgramImage,
    state: &EpactRuntimeState,
    request: &EpactOperationRequest,
) -> Result<EpactEligibility, EpactRuntimeError> {
    require_activatable(image)
        .map_err(|error| EpactRuntimeError::InvalidImage(error.to_string()))?;
    validate_state_shape(image, state)?;

    let mut blockers = Vec::new();
    let request_time_valid = validate_epact_timestamp(&request.requested_at);
    if !request_time_valid {
        blocker(
            &mut blockers,
            "invalid_request_time",
            &image.program.id,
            "request time must use canonical Epact UTC-second form",
        );
    }
    let principal_known = image
        .program
        .principals
        .iter()
        .any(|principal| principal.id == request.principal_id);
    if !principal_known {
        blocker(
            &mut blockers,
            "unknown_principal",
            &request.principal_id,
            "the requested principal is not declared by this program",
        );
    }

    let obligation = request.obligation_id.as_deref().and_then(|id| {
        image
            .program
            .obligations
            .iter()
            .find(|obligation| obligation.id == id)
    });
    if let Some(id) = &request.obligation_id {
        if obligation.is_none() {
            blocker(
                &mut blockers,
                "unknown_obligation",
                id,
                "the requested obligation is not declared by this program",
            );
        }
    } else if operation_requires_obligation(request.operation) {
        blocker(
            &mut blockers,
            "obligation_required",
            &image.program.id,
            "this operation must be bound to an obligation",
        );
    }

    let capability_known = request.capability_id.as_deref().is_none_or(|id| {
        image
            .program
            .capabilities
            .iter()
            .any(|capability| capability.id == id)
    });
    if !capability_known {
        blocker(
            &mut blockers,
            "unknown_capability",
            request.capability_id.as_deref().unwrap_or_default(),
            "the requested capability is not declared by this program",
        );
    }

    if !request.resources.is_finite_and_non_negative() {
        blocker(
            &mut blockers,
            "invalid_resources",
            &image.program.id,
            "requested resource values must be finite and non-negative",
        );
    } else if !request.resources.fits_within(&image.program.resources) {
        blocker(
            &mut blockers,
            "program_resource_ceiling",
            &image.program.id,
            "requested resources exceed the frozen program ceiling",
        );
    }

    if let Some(obligation) = obligation {
        evaluate_obligation_request(image, state, request, obligation, &mut blockers);
    }

    if request_time_valid
        && principal_known
        && capability_known
        && !authority_allows(image, request, &request.resources)
    {
        blocker(
            &mut blockers,
            "authority_denied",
            &request.principal_id,
            "no compiled authority covers this operation, scope, and cost",
        );
    }

    blockers.sort_by(|left, right| {
        (&left.code, &left.subject_id, &left.message).cmp(&(
            &right.code,
            &right.subject_id,
            &right.message,
        ))
    });
    blockers.dedup();
    Ok(EpactEligibility {
        allowed: blockers.is_empty(),
        blockers,
    })
}

/// True only when all declared terminal obligations, objects, and receipt contracts are present.
pub fn epact_program_is_terminal(
    image: &EpactProgramImage,
    state: &EpactRuntimeState,
    events: &[EpactRuntimeEvent],
) -> Result<bool, EpactRuntimeError> {
    validate_state_shape(image, state)?;
    let satisfied = state
        .obligations
        .iter()
        .filter(|projection| projection.state == EpactObligationState::Satisfied)
        .map(|projection| projection.obligation_id.as_str())
        .collect::<BTreeSet<_>>();
    let objects = state
        .present_object_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let receipts = events
        .iter()
        .filter_map(|event| match &event.kind {
            EpactRuntimeEventKind::ObligationSatisfied {
                receipt_contract, ..
            } => Some(receipt_contract.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    Ok(image
        .program
        .terminal
        .required_obligation_ids
        .iter()
        .all(|id| satisfied.contains(id.as_str()))
        && image
            .program
            .terminal
            .required_object_ids
            .iter()
            .all(|id| objects.contains(id.as_str()))
        && image
            .program
            .terminal
            .required_receipt_contracts
            .iter()
            .all(|contract| receipts.contains(contract.as_str())))
}

fn apply_event(
    image: &EpactProgramImage,
    state: &mut EpactRuntimeState,
    event: &EpactRuntimeEvent,
) -> Result<(), EpactRuntimeError> {
    match &event.kind {
        EpactRuntimeEventKind::ObjectRecorded { object_id } => {
            if !image
                .program
                .objects
                .iter()
                .any(|object| object.id == *object_id)
            {
                return Err(EpactRuntimeError::UnknownObject(object_id.clone()));
            }
            insert_sorted_unique(&mut state.present_object_ids, object_id.clone());
        }
        EpactRuntimeEventKind::EvidenceAccepted {
            evidence_rule_id,
            independent_review_receipt_sha256,
        } => {
            let rule = image
                .program
                .evidence_rules
                .iter()
                .find(|rule| rule.id == *evidence_rule_id)
                .ok_or_else(|| EpactRuntimeError::UnknownEvidenceRule(evidence_rule_id.clone()))?;
            let observation_count = rule
                .evidence_object_ids
                .iter()
                .filter(|id| state.present_object_ids.binary_search(id).is_ok())
                .count();
            if observation_count < rule.minimum_observations as usize {
                return Err(EpactRuntimeError::InsufficientEvidence(
                    evidence_rule_id.clone(),
                ));
            }
            if rule.independent_review_required && independent_review_receipt_sha256.is_none() {
                return Err(EpactRuntimeError::IndependentReviewRequired(
                    evidence_rule_id.clone(),
                ));
            }
            insert_sorted_unique(
                &mut state.satisfied_evidence_rule_ids,
                evidence_rule_id.clone(),
            );
        }
        EpactRuntimeEventKind::ReviewAccepted {
            obligation_id,
            review_object_id,
            reviewer_principal_id,
            independent_review_receipt_sha256,
        } => {
            let obligation = find_obligation(image, obligation_id)?;
            let review = find_review_discharge(&obligation.discharge, review_object_id)
                .ok_or_else(|| EpactRuntimeError::UnknownReviewPath(obligation_id.clone()))?;
            if !image
                .program
                .principals
                .iter()
                .any(|principal| principal.id == *reviewer_principal_id)
            {
                return Err(EpactRuntimeError::UnknownPrincipal(
                    reviewer_principal_id.clone(),
                ));
            }
            if review && reviewer_principal_id == &event.actor {
                return Err(EpactRuntimeError::IndependentReviewerRequired(
                    obligation_id.clone(),
                ));
            }
            if state
                .present_object_ids
                .binary_search(review_object_id)
                .is_err()
            {
                return Err(EpactRuntimeError::MissingDischargeObject {
                    obligation_id: obligation_id.clone(),
                    object_id: review_object_id.clone(),
                });
            }
            insert_sorted_unique(
                &mut state.accepted_reviews,
                EpactAcceptedReview {
                    obligation_id: obligation_id.clone(),
                    review_object_id: review_object_id.clone(),
                    reviewer_principal_id: reviewer_principal_id.clone(),
                    independent_review_receipt_sha256: independent_review_receipt_sha256.clone(),
                },
            );
        }
        EpactRuntimeEventKind::ObligationSatisfied {
            obligation_id,
            receipt_contract,
        } => {
            let obligation = find_obligation(image, obligation_id)?;
            require_obligation_pending(state, obligation_id)?;
            require_dependencies(state, obligation)?;
            require_gates(image, state, obligation)?;
            require_discharge(state, obligation)?;
            if receipt_contract != &obligation.terminal_receipt_contract {
                return Err(EpactRuntimeError::ReceiptContractMismatch {
                    obligation_id: obligation_id.clone(),
                    expected: obligation.terminal_receipt_contract.clone(),
                    actual: receipt_contract.clone(),
                });
            }
            set_terminal_state(
                state,
                obligation_id,
                EpactObligationState::Satisfied,
                &event.event_sha256,
            )?;
        }
        EpactRuntimeEventKind::ObligationFailed { obligation_id, .. } => {
            find_obligation(image, obligation_id)?;
            require_obligation_pending(state, obligation_id)?;
            set_terminal_state(
                state,
                obligation_id,
                EpactObligationState::Failed,
                &event.event_sha256,
            )?;
        }
        EpactRuntimeEventKind::ObligationCancelled { obligation_id, .. } => {
            find_obligation(image, obligation_id)?;
            require_obligation_pending(state, obligation_id)?;
            set_terminal_state(
                state,
                obligation_id,
                EpactObligationState::Cancelled,
                &event.event_sha256,
            )?;
        }
    }
    Ok(())
}

fn evaluate_obligation_request(
    image: &EpactProgramImage,
    state: &EpactRuntimeState,
    request: &EpactOperationRequest,
    obligation: &EpactObligation,
    blockers: &mut Vec<EpactEligibilityBlocker>,
) {
    let projection = state
        .obligations
        .iter()
        .find(|projection| projection.obligation_id == obligation.id);
    if !projection.is_some_and(|projection| projection.state == EpactObligationState::Pending) {
        blocker(
            blockers,
            "obligation_not_pending",
            &obligation.id,
            "the obligation has already reached a terminal state",
        );
    }
    if !request.resources.fits_within(&obligation.resources) {
        blocker(
            blockers,
            "obligation_resource_ceiling",
            &obligation.id,
            "requested resources exceed the obligation ceiling",
        );
    }

    let mut requested_effects = request.effects.clone();
    requested_effects.sort();
    requested_effects.dedup();
    if requested_effects != obligation.effects {
        blocker(
            blockers,
            "effect_mismatch",
            &obligation.id,
            "requested effects must exactly match the frozen obligation declaration",
        );
    }

    let expected_capabilities = discharge_capability_ids(&obligation.discharge, request.operation);
    if !expected_capabilities.is_empty() {
        if !request
            .capability_id
            .as_deref()
            .is_some_and(|candidate| expected_capabilities.contains(&candidate))
        {
            blocker(
                blockers,
                "capability_mismatch",
                &obligation.id,
                "requested capability does not discharge this obligation",
            );
        }
    } else if request.capability_id.is_some() {
        blocker(
            blockers,
            "unexpected_capability",
            &obligation.id,
            "this obligation is not discharged by a capability",
        );
    }

    if let Some(capability_id) = request.capability_id.as_deref() {
        if let Some(capability) = image
            .program
            .capabilities
            .iter()
            .find(|capability| capability.id == capability_id)
        {
            evaluate_placement_request(capability, request, blockers);
        }
    } else if request.placement.is_some() {
        blocker(
            blockers,
            "unexpected_placement",
            &obligation.id,
            "placement may only bind a declared capability",
        );
    }

    if operation_requires_ready_obligation(request.operation) {
        for dependency in &obligation.dependency_ids {
            if obligation_state(state, dependency) != Some(EpactObligationState::Satisfied) {
                blocker(
                    blockers,
                    "dependency_unsatisfied",
                    dependency,
                    "a required predecessor obligation is not satisfied",
                );
            }
        }
        for gate_id in &obligation.gate_ids {
            if !gate_satisfied(image, state, gate_id) {
                blocker(
                    blockers,
                    "gate_unsatisfied",
                    gate_id,
                    "a required gate predicate is false",
                );
            }
        }
    }
}

fn evaluate_placement_request(
    capability: &epact_protocol::EpactCapabilityRequirement,
    request: &EpactOperationRequest,
    blockers: &mut Vec<EpactEligibilityBlocker>,
) {
    let Some(policy) = &capability.placement else {
        return;
    };
    let Some(claim) = &request.placement else {
        blocker(
            blockers,
            "placement_required",
            &capability.id,
            "this capability requires a qualified placement claim",
        );
        return;
    };
    if !policy.allowed_kinds.contains(&claim.kind) {
        blocker(
            blockers,
            "placement_kind_denied",
            &capability.id,
            "the selected placement kind is not admitted by this capability",
        );
    }
    if !is_sorted_unique(&claim.target_capabilities) {
        blocker(
            blockers,
            "noncanonical_placement_capabilities",
            &capability.id,
            "placement target capabilities must be sorted and unique",
        );
    } else if !policy
        .required_target_capabilities
        .iter()
        .all(|required| claim.target_capabilities.binary_search(required).is_ok())
    {
        blocker(
            blockers,
            "placement_capability_missing",
            &capability.id,
            "the selected target lacks a required placement capability",
        );
    }
    if policy.requires_disconnect_safety && !claim.disconnect_safe {
        blocker(
            blockers,
            "disconnect_safety_required",
            &capability.id,
            "the selected target cannot survive operator disconnection",
        );
    }
}

fn authority_allows(
    image: &EpactProgramImage,
    request: &EpactOperationRequest,
    resources: &EpactResourceEnvelope,
) -> bool {
    image.authorities.iter().any(|authority| {
        authority.principal_id == request.principal_id
            && authority.operation == request.operation
            && authority_scope_matches(authority, request)
            && authority_cost_allows(authority, resources.maximum_cost_usd)
            && authority_time_allows(authority, &request.requested_at)
    })
}

fn authority_time_allows(authority: &CompiledAuthority, requested_at: &str) -> bool {
    authority
        .valid_after
        .as_ref()
        .is_none_or(|after| requested_at >= after)
        && authority
            .valid_before
            .as_ref()
            .is_none_or(|before| requested_at < before)
}

fn authority_scope_matches(authority: &CompiledAuthority, request: &EpactOperationRequest) -> bool {
    authority.whole_program
        || request.obligation_id.as_ref().is_some_and(|id| {
            authority
                .obligation_ids
                .iter()
                .any(|candidate| candidate == id)
        })
        || request.capability_id.as_ref().is_some_and(|id| {
            authority
                .capability_ids
                .iter()
                .any(|candidate| candidate == id)
        })
}

fn authority_cost_allows(authority: &CompiledAuthority, requested_cost_usd: f64) -> bool {
    if requested_cost_usd <= 0.0 {
        return true;
    }
    authority
        .maximum_cost_microusd
        .is_some_and(|ceiling| (requested_cost_usd * 1_000_000.0).round() as u64 <= ceiling)
}

fn operation_requires_obligation(operation: KernelOperation) -> bool {
    matches!(
        operation,
        KernelOperation::Propose
            | KernelOperation::Authorize
            | KernelOperation::Reserve
            | KernelOperation::Dispatch
            | KernelOperation::Attest
            | KernelOperation::Evaluate
            | KernelOperation::Decide
            | KernelOperation::Publish
            | KernelOperation::Retract
    )
}

fn operation_requires_ready_obligation(operation: KernelOperation) -> bool {
    matches!(
        operation,
        KernelOperation::Reserve
            | KernelOperation::Dispatch
            | KernelOperation::Evaluate
            | KernelOperation::Decide
            | KernelOperation::Publish
            | KernelOperation::Retract
    )
}

fn discharge_capability_ids(
    discharge: &EpactDischarge,
    operation: KernelOperation,
) -> BTreeSet<&str> {
    match discharge {
        EpactDischarge::AnyOf { alternatives } => alternatives
            .iter()
            .flat_map(|alternative| discharge_capability_ids(alternative, operation))
            .collect(),
        EpactDischarge::Capability { capability_id }
            if matches!(
                operation,
                KernelOperation::Propose
                    | KernelOperation::Authorize
                    | KernelOperation::Reserve
                    | KernelOperation::Dispatch
            ) =>
        {
            BTreeSet::from([capability_id.as_str()])
        }
        EpactDischarge::Review { capability_id, .. }
            if matches!(
                operation,
                KernelOperation::Propose
                    | KernelOperation::Authorize
                    | KernelOperation::Reserve
                    | KernelOperation::Evaluate
            ) =>
        {
            BTreeSet::from([capability_id.as_str()])
        }
        _ => BTreeSet::new(),
    }
}

fn validate_state_shape(
    image: &EpactProgramImage,
    state: &EpactRuntimeState,
) -> Result<(), EpactRuntimeError> {
    if state.program_image_sha256 != image.image_sha256 {
        return Err(EpactRuntimeError::ImageBindingMismatch);
    }
    let expected = image
        .obligation_order
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual = state
        .obligations
        .iter()
        .map(|projection| projection.obligation_id.as_str())
        .collect::<BTreeSet<_>>();
    if expected != actual || actual.len() != state.obligations.len() {
        return Err(EpactRuntimeError::InvalidState(
            "obligation projection does not match compiled image",
        ));
    }
    if !is_sorted_unique(&state.present_object_ids)
        || !is_sorted_unique(&state.satisfied_evidence_rule_ids)
        || !is_sorted_unique(&state.accepted_reviews)
    {
        return Err(EpactRuntimeError::InvalidState(
            "set projections must be sorted and unique",
        ));
    }
    Ok(())
}

fn require_obligation_pending(
    state: &EpactRuntimeState,
    obligation_id: &str,
) -> Result<(), EpactRuntimeError> {
    if obligation_state(state, obligation_id) != Some(EpactObligationState::Pending) {
        return Err(EpactRuntimeError::ObligationAlreadyTerminal(
            obligation_id.to_owned(),
        ));
    }
    Ok(())
}

fn require_dependencies(
    state: &EpactRuntimeState,
    obligation: &EpactObligation,
) -> Result<(), EpactRuntimeError> {
    for dependency in &obligation.dependency_ids {
        if obligation_state(state, dependency) != Some(EpactObligationState::Satisfied) {
            return Err(EpactRuntimeError::UnsatisfiedDependency {
                obligation_id: obligation.id.clone(),
                dependency_id: dependency.clone(),
            });
        }
    }
    Ok(())
}

fn require_gates(
    image: &EpactProgramImage,
    state: &EpactRuntimeState,
    obligation: &EpactObligation,
) -> Result<(), EpactRuntimeError> {
    for gate_id in &obligation.gate_ids {
        if !gate_satisfied(image, state, gate_id) {
            return Err(EpactRuntimeError::UnsatisfiedGate {
                obligation_id: obligation.id.clone(),
                gate_id: gate_id.clone(),
            });
        }
    }
    Ok(())
}

fn require_discharge(
    state: &EpactRuntimeState,
    obligation: &EpactObligation,
) -> Result<(), EpactRuntimeError> {
    for object_id in &obligation.output_object_ids {
        if state.present_object_ids.binary_search(&object_id).is_err() {
            return Err(EpactRuntimeError::MissingDischargeObject {
                obligation_id: obligation.id.clone(),
                object_id: object_id.clone(),
            });
        }
    }
    if !discharge_satisfied(state, &obligation.discharge, &obligation.id) {
        return Err(EpactRuntimeError::UnsatisfiedDischarge(
            obligation.id.clone(),
        ));
    }
    Ok(())
}

fn discharge_satisfied(
    state: &EpactRuntimeState,
    discharge: &EpactDischarge,
    obligation_id: &str,
) -> bool {
    match discharge {
        EpactDischarge::AnyOf { alternatives } => alternatives
            .iter()
            .any(|alternative| discharge_satisfied(state, alternative, obligation_id)),
        EpactDischarge::Capability { .. } => true,
        EpactDischarge::Decision { decision_object_id } => state
            .present_object_ids
            .binary_search(decision_object_id)
            .is_ok(),
        EpactDischarge::Evidence { evidence_rule_ids } => evidence_rule_ids.iter().all(|rule_id| {
            state
                .satisfied_evidence_rule_ids
                .binary_search(rule_id)
                .is_ok()
        }),
        EpactDischarge::Review {
            review_object_id, ..
        } => {
            state
                .present_object_ids
                .binary_search(review_object_id)
                .is_ok()
                && state.accepted_reviews.iter().any(|accepted| {
                    accepted.obligation_id == obligation_id
                        && accepted.review_object_id == *review_object_id
                })
        }
        EpactDischarge::Publication {
            artifact_object_ids,
        } => artifact_object_ids
            .iter()
            .all(|object_id| state.present_object_ids.binary_search(object_id).is_ok()),
    }
}

fn find_review_discharge(discharge: &EpactDischarge, review_object_id: &str) -> Option<bool> {
    match discharge {
        EpactDischarge::AnyOf { alternatives } => alternatives
            .iter()
            .find_map(|alternative| find_review_discharge(alternative, review_object_id)),
        EpactDischarge::Review {
            review_object_id: expected,
            independent_principal_required,
            ..
        } if expected == review_object_id => Some(*independent_principal_required),
        _ => None,
    }
}

fn gate_satisfied(image: &EpactProgramImage, state: &EpactRuntimeState, gate_id: &str) -> bool {
    image
        .program
        .gates
        .iter()
        .find(|gate| gate.id == gate_id)
        .is_some_and(|gate| predicate_satisfied(&gate.predicate, state))
}

fn predicate_satisfied(predicate: &EpactPredicate, state: &EpactRuntimeState) -> bool {
    match predicate {
        EpactPredicate::All { predicates } => predicates
            .iter()
            .all(|predicate| predicate_satisfied(predicate, state)),
        EpactPredicate::Any { predicates } => predicates
            .iter()
            .any(|predicate| predicate_satisfied(predicate, state)),
        EpactPredicate::Not { predicate } => !predicate_satisfied(predicate, state),
        EpactPredicate::ObligationSatisfied { obligation_id } => {
            obligation_state(state, obligation_id) == Some(EpactObligationState::Satisfied)
        }
        EpactPredicate::EvidenceSatisfied { evidence_rule_id } => state
            .satisfied_evidence_rule_ids
            .binary_search(evidence_rule_id)
            .is_ok(),
        EpactPredicate::ObjectPresent { object_id } => {
            state.present_object_ids.binary_search(object_id).is_ok()
        }
    }
}

fn find_obligation<'a>(
    image: &'a EpactProgramImage,
    obligation_id: &str,
) -> Result<&'a EpactObligation, EpactRuntimeError> {
    image
        .program
        .obligations
        .iter()
        .find(|obligation| obligation.id == obligation_id)
        .ok_or_else(|| EpactRuntimeError::UnknownObligation(obligation_id.to_owned()))
}

fn obligation_state(
    state: &EpactRuntimeState,
    obligation_id: &str,
) -> Option<EpactObligationState> {
    state
        .obligations
        .iter()
        .find(|projection| projection.obligation_id == obligation_id)
        .map(|projection| projection.state)
}

fn set_terminal_state(
    state: &mut EpactRuntimeState,
    obligation_id: &str,
    terminal: EpactObligationState,
    event_sha256: &str,
) -> Result<(), EpactRuntimeError> {
    let projection = state
        .obligations
        .iter_mut()
        .find(|projection| projection.obligation_id == obligation_id)
        .ok_or_else(|| EpactRuntimeError::UnknownObligation(obligation_id.to_owned()))?;
    projection.state = terminal;
    projection.terminal_event_sha256 = Some(event_sha256.to_owned());
    Ok(())
}

fn insert_sorted_unique<T: Ord>(values: &mut Vec<T>, value: T) {
    match values.binary_search(&value) {
        Ok(_) => {}
        Err(index) => values.insert(index, value),
    }
}

fn is_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|window| window[0] < window[1])
}

fn blocker(
    blockers: &mut Vec<EpactEligibilityBlocker>,
    code: &str,
    subject_id: &str,
    message: &str,
) {
    blockers.push(EpactEligibilityBlocker {
        code: code.to_owned(),
        subject_id: subject_id.to_owned(),
        message: message.to_owned(),
    });
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EpactRuntimeError {
    #[error("invalid Epact program image: {0}")]
    InvalidImage(String),
    #[error("invalid Epact runtime event: {0}")]
    InvalidEvent(String),
    #[error("runtime state or event is bound to another program image")]
    ImageBindingMismatch,
    #[error("expected event sequence {expected}, found {actual}")]
    UnexpectedSequence { expected: u64, actual: u64 },
    #[error("event {0} does not extend the current event hash")]
    BrokenEventChain(String),
    #[error("duplicate event id {0}")]
    DuplicateEventId(String),
    #[error("duplicate idempotency key {0}")]
    DuplicateIdempotencyKey(String),
    #[error("unknown principal {0}")]
    UnknownPrincipal(String),
    #[error("event {0} has no declared authority path")]
    EventAuthorityPathUnavailable(String),
    #[error("principal {actor} lacks authority to append event {event_id}")]
    EventAuthorityDenied { event_id: String, actor: String },
    #[error("unknown object {0}")]
    UnknownObject(String),
    #[error("unknown evidence rule {0}")]
    UnknownEvidenceRule(String),
    #[error("unknown obligation {0}")]
    UnknownObligation(String),
    #[error("obligation {0} does not declare the accepted review path")]
    UnknownReviewPath(String),
    #[error("evidence rule {0} has too few recorded observations")]
    InsufficientEvidence(String),
    #[error("evidence rule {0} requires an independent-review receipt")]
    IndependentReviewRequired(String),
    #[error("obligation {0} requires a reviewer distinct from the accepting actor")]
    IndependentReviewerRequired(String),
    #[error("obligation {0} already reached a terminal state")]
    ObligationAlreadyTerminal(String),
    #[error("obligation {obligation_id} requires unsatisfied dependency {dependency_id}")]
    UnsatisfiedDependency {
        obligation_id: String,
        dependency_id: String,
    },
    #[error("obligation {obligation_id} requires unsatisfied gate {gate_id}")]
    UnsatisfiedGate {
        obligation_id: String,
        gate_id: String,
    },
    #[error("obligation {obligation_id} requires missing object {object_id}")]
    MissingDischargeObject {
        obligation_id: String,
        object_id: String,
    },
    #[error("obligation {obligation_id} requires unsatisfied evidence rule {evidence_rule_id}")]
    UnsatisfiedEvidence {
        obligation_id: String,
        evidence_rule_id: String,
    },
    #[error("obligation {0} has no satisfied discharge alternative")]
    UnsatisfiedDischarge(String),
    #[error("obligation {obligation_id} requires receipt contract {expected}, found {actual}")]
    ReceiptContractMismatch {
        obligation_id: String,
        expected: String,
        actual: String,
    },
    #[error("invalid Epact runtime state: {0}")]
    InvalidState(&'static str),
}
