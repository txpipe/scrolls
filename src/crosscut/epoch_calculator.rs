// TODO this is temporary, we should actually use this code from Pallas as this is very generic code
pub struct EpochCalculator {
}

impl EpochCalculator {
    
    pub fn get_shelley_epoch_no_for_absolute_slot(
        shelley_known_slot: u64,
        shelley_epoch_length: u64,
        slot: u64
    ) -> u64 {
        let shelley_known_slot = shelley_known_slot as u64;
        let shelley_epoch_length = shelley_epoch_length as u64;

        let shelley_epoch_no = (slot - shelley_known_slot) / shelley_epoch_length;
        let last_byron_epoch_no = 208;

        return last_byron_epoch_no + shelley_epoch_no;
    }

    pub fn get_byron_epoch_no_for_absolute_slot(
        byron_epoch_length: u64,
        byron_slot_length: u64, 
        slot: u64,
    ) -> u64 {
        let byron_epoch_length = byron_epoch_length as u64;
        let byron_slot_length = byron_slot_length as u64;

        return slot / (byron_epoch_length / byron_slot_length)
    }

}
