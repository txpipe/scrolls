use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ReducerPolicy {
    skip_missing_data: Option<bool>,
    skip_cbor_errors: Option<bool>,
    skip_ledger_errors: Option<bool>,
    skip_all_errors: Option<bool>,
}

pub trait AppliesPolicy {
    type Value;

    fn apply_policy(
        self,
        policy: &Option<ReducerPolicy>,
    ) -> Result<Option<Self::Value>, crate::Error>;
}

impl<T> AppliesPolicy for Result<T, crate::Error> {
    type Value = T;

    fn apply_policy(
        self,
        policy: &Option<ReducerPolicy>,
    ) -> Result<Option<Self::Value>, crate::Error> {
        match self {
            Ok(t) => Ok(Some(t)),
            Err(err) => {
                if matches!(policy, None) {
                    return Err(err);
                }

                let policy = policy.as_ref().unwrap();

                if policy.skip_all_errors.unwrap_or(false) {
                    return Ok(None);
                }

                match &err {
                    crate::Error::MissingTx(_) if policy.skip_missing_data.unwrap_or(false) => {
                        log::warn!("{}", err);
                        Ok(None)
                    }
                    crate::Error::CborError(_) if policy.skip_cbor_errors.unwrap_or(false) => {
                        log::warn!("{}", err);
                        Ok(None)
                    }
                    crate::Error::LedgerError(_) if policy.skip_ledger_errors.unwrap_or(false) => {
                        log::warn!("{}", err);
                        Ok(None)
                    }
                    _ => Err(err),
                }
            }
        }
    }
}
