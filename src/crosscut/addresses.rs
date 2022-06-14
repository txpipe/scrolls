fn get_bit_at(input: u8, n: u8) -> bool {
    if n < 32 {
        input & (1 << n) != 0
    } else {
        false
    }
}

// https://github.com/input-output-hk/cardano-ledger/blob/master/eras/alonzo/test-suite/cddl-files/alonzo.cddl#L135
pub fn is_smart_contract(address: &[u8]) -> bool {
    let byte_1 = address[0];
    return get_bit_at(byte_1, 4);
}
