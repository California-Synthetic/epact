use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

/// The externally observable class of change an Epact capability may cause.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EffectClass {
    ReadOnly,
    LocalWrite,
    NetworkRead,
    ExternalWrite,
    PaidCompute,
    RestrictedData,
    Instrument,
}

/// How a completed effect can be unwound, superseded, or only acknowledged.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityClass {
    #[default]
    Unspecified,
    ReadOnly,
    AppendOnly,
    CheckpointRestore,
    CompensatingAction,
    Irreversible,
}

/// Portable unwind semantics for an effect.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReversibilityPolicy {
    pub class: ReversibilityClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reversal_action: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
}

impl ReversibilityPolicy {
    pub fn is_unspecified(&self) -> bool {
        self.class == ReversibilityClass::Unspecified
            && self.reversal_action.is_none()
            && self.limitations.is_empty()
    }

    pub fn validate(&self, effect: EffectClass) -> Result<(), EffectPolicyError> {
        if self.is_unspecified() {
            return Ok(());
        }
        if matches!(effect, EffectClass::ReadOnly | EffectClass::NetworkRead)
            && self.class != ReversibilityClass::ReadOnly
        {
            return Err(EffectPolicyError::ReadRequiresReadOnly);
        }
        if matches!(self.class, ReversibilityClass::ReadOnly)
            && !matches!(effect, EffectClass::ReadOnly | EffectClass::NetworkRead)
        {
            return Err(EffectPolicyError::EffectCannotClaimReadOnly);
        }
        if !matches!(
            self.class,
            ReversibilityClass::ReadOnly | ReversibilityClass::Unspecified
        ) && self
            .reversal_action
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(EffectPolicyError::MissingReversalAction);
        }
        if self.class == ReversibilityClass::Irreversible && self.limitations.is_empty() {
            return Err(EffectPolicyError::MissingIrreversibleLimitation);
        }
        if let Some(action) = &self.reversal_action {
            if action.trim().is_empty() || action.len() > 2_000 {
                return Err(EffectPolicyError::InvalidReversalAction);
            }
        }
        let mut limitations = BTreeSet::new();
        for limitation in &self.limitations {
            if limitation.trim().is_empty() || limitation.len() > 1_000 {
                return Err(EffectPolicyError::InvalidLimitation);
            }
            if !limitations.insert(limitation) {
                return Err(EffectPolicyError::DuplicateLimitation);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum EffectPolicyError {
    #[error("read-only effects require read-only reversibility")]
    ReadRequiresReadOnly,
    #[error("effectful tools cannot claim read-only reversibility")]
    EffectCannotClaimReadOnly,
    #[error("effectful reversibility policies require a reversal or compensation action")]
    MissingReversalAction,
    #[error("irreversible actions require an explicit residual-effect limitation")]
    MissingIrreversibleLimitation,
    #[error("reversal action must contain 1-2000 characters")]
    InvalidReversalAction,
    #[error("reversibility limitations must contain 1-1000 characters")]
    InvalidLimitation,
    #[error("duplicate reversibility limitation")]
    DuplicateLimitation,
}
