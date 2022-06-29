use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ErrorAction {
    Skip,
    Warn,
    Default,
}

impl Default for ErrorAction {
    fn default() -> Self {
        ErrorAction::Default
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct RuntimePolicy {
    missing_data: Option<ErrorAction>,
    cbor_errors: Option<ErrorAction>,
    ledger_errors: Option<ErrorAction>,
    any_error: Option<ErrorAction>,
}

#[inline]
fn handle_error<T>(err: Error, action: &Option<ErrorAction>) -> Result<Option<T>, crate::Error> {
    match action {
        Some(ErrorAction::Skip) => Ok(None),
        Some(ErrorAction::Warn) => {
            log::warn!("{}", err);
            Ok(None)
        }
        _ => Err(err),
    }
}

pub trait AppliesPolicy {
    type Value;

    fn apply_policy(self, policy: &RuntimePolicy) -> Result<Option<Self::Value>, crate::Error>;
}

impl<T> AppliesPolicy for Result<T, crate::Error> {
    type Value = T;

    fn apply_policy(self, policy: &RuntimePolicy) -> Result<Option<Self::Value>, crate::Error> {
        match self {
            Ok(t) => Ok(Some(t)),
            Err(err) => {
                // apply generic error policy if we have one
                if matches!(policy.any_error, Some(_)) {
                    return handle_error(err, &policy.any_error);
                }

                // apply specific actions for each type of error
                match &err {
                    crate::Error::MissingUtxo(_) => handle_error(err, &policy.missing_data),
                    crate::Error::CborError(_) => handle_error(err, &policy.cbor_errors),
                    crate::Error::LedgerError(_) => handle_error(err, &policy.ledger_errors),
                    _ => Err(err),
                }
            }
        }
    }
}
