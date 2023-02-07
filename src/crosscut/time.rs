use super::ChainWellKnownInfo;

#[inline]
fn compute_linear_timestamp(
    known_slot: u64,
    known_time: u64,
    slot_length: u64,
    query_slot: u64,
) -> u64 {
    known_time + (query_slot - known_slot) * slot_length
}

#[inline]
fn compute_era_epoch(era_slot: u64, era_slot_length: u64, era_epoch_length: u64) -> (u64, u64) {
    let epoch = (era_slot * era_slot_length) / era_epoch_length;
    let reminder = era_slot % era_epoch_length;

    (epoch, reminder)
}

/// A naive, standalone implementation of a time provider
///
/// This time provider doesn't require any external resources other than an
/// initial config. It works by applying simple slot => wallclock conversion
/// logic from a well-known configured point in the chain, assuming homogeneous
/// slot length from that point forward.
#[derive(Clone)]
pub(crate) struct NaiveProvider {
    config: ChainWellKnownInfo,
    shelley_start_epoch: u64,
}

impl NaiveProvider {
    pub fn new(config: ChainWellKnownInfo) -> Self {
        assert!(
            config.byron_epoch_length > 0,
            "byron epoch length needs to be greater than zero"
        );

        assert!(
            config.shelley_epoch_length > 0,
            "shelley epoch length needs to be greater than zero"
        );

        let (shelley_start_epoch, _) = compute_era_epoch(
            config.shelley_known_slot,
            config.byron_slot_length as u64,
            config.byron_epoch_length as u64,
        );

        NaiveProvider {
            config,
            shelley_start_epoch,
        }
    }

    pub fn slot_to_wallclock(&self, slot: u64) -> u64 {
        let NaiveProvider { config, .. } = self;

        if slot < config.shelley_known_slot {
            compute_linear_timestamp(
                config.byron_known_slot,
                config.byron_known_time,
                config.byron_slot_length as u64,
                slot,
            )
        } else {
            compute_linear_timestamp(
                config.shelley_known_slot,
                config.shelley_known_time,
                config.shelley_slot_length as u64,
                slot,
            )
        }
    }
}
