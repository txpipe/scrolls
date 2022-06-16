// TODO this is temporary, we should actually use this code from Pallas as this
// is very generic code

use pallas::ledger::traverse::MultiEraBlock;

fn post_byron_epoch_for_slot(shelley_known_slot: u64, shelley_epoch_length: u32, slot: u64) -> u64 {
    let last_byron_epoch_no = 208;

    let shelley_known_slot = shelley_known_slot as u64;
    let shelley_epoch_length = shelley_epoch_length as u64;

    let shelley_epoch_no = (slot - shelley_known_slot) / shelley_epoch_length;

    return last_byron_epoch_no + shelley_epoch_no;
}

fn byron_epoch_for_slot(byron_epoch_length: u32, byron_slot_length: u32, slot: u64) -> u64 {
    let byron_epoch_length = byron_epoch_length as u64;
    let byron_slot_length = byron_slot_length as u64;

    return slot / (byron_epoch_length / byron_slot_length);
}

pub fn block_epoch(chain: &super::ChainWellKnownInfo, block: &MultiEraBlock) -> u64 {
    let slot = block.slot();

    match block.era() {
        pallas::ledger::traverse::Era::Byron => {
            byron_epoch_for_slot(chain.byron_epoch_length, chain.byron_slot_length, slot)
        }
        _ => post_byron_epoch_for_slot(chain.shelley_known_slot, chain.shelley_epoch_length, slot),
    }
}
