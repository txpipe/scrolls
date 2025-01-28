use pallas::ledger::{
    addresses::Address,
    traverse::{MultiEraBlock, MultiEraTx},
};
use serde::Deserialize;

use crate::{
    crosscut,
    framework::{errors::Error, model},
};

use super::policies::AppliesPolicy;

#[derive(Deserialize, Clone, Default)]
pub struct AddressPattern {
    pub exact_hex: Option<String>,
    pub exact_bech32: Option<String>,
    pub payment_hex: Option<String>,
    pub payment_bech32: Option<String>,
    pub stake_hex: Option<String>,
    pub stake_bech32: Option<String>,
    pub is_script: Option<bool>,
}

impl AddressPattern {
    pub fn matches(&self, addr: Address) -> bool {
        if let Some(x) = &self.exact_hex {
            if addr.to_hex().eq(x) {
                return true;
            }
        }

        if let Some(x) = &self.exact_bech32 {
            if let Ok(addr) = addr.to_bech32() {
                if addr.eq(x) {
                    return true;
                }
            }
        }

        if let Some(x) = &self.payment_hex {
            if let Address::Shelley(ref a) = addr {
                let payment_hex = a.payment().to_hex();

                if payment_hex.eq(x) {
                    return true;
                }
            }
        }

        if let Some(x) = &self.payment_bech32 {
            if let Address::Shelley(ref a) = addr {
                if let Ok(payment_bech32) = a.payment().to_bech32() {
                    if payment_bech32.eq(x) {
                        return true;
                    }
                }
            }
        }

        if let Some(x) = &self.stake_hex {
            if let Address::Shelley(ref a) = addr {
                let delegation_hex = a.delegation().to_hex();

                if delegation_hex.eq(x) {
                    return true;
                }
            }
        }

        if let Some(x) = &self.stake_bech32 {
            if let Address::Shelley(ref a) = addr {
                if let Ok(deleg_bech32) = a.delegation().to_bech32() {
                    if deleg_bech32.eq(x) {
                        return true;
                    }
                }
            }
        }

        if let Some(x) = &self.is_script {
            return addr.has_script() == *x;
        }

        false
    }
}

#[derive(Deserialize, Clone)]
pub struct BlockPattern {
    pub slot_before: Option<u64>,
    pub slot_after: Option<u64>,
}

#[derive(Deserialize, Clone)]
pub struct TransactionPattern {
    pub is_valid: Option<bool>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    AllOf(Vec<Predicate>),
    AnyOf(Vec<Predicate>),
    Not(Box<Predicate>),
    Block(BlockPattern),
    Transaction(TransactionPattern),
    InputAddress(AddressPattern),
    OutputAddress(AddressPattern),
    WithdrawalAddress(AddressPattern),
    CollateralAddress(AddressPattern),

    /// Filters by an address referenced in any part of the tx
    Address(AddressPattern),
}

impl Predicate {
    pub fn and(&self, other: &Self) -> Self {
        Predicate::AllOf(vec![self.clone(), other.clone()])
    }
}

#[inline]
fn eval_output_address(tx: &MultiEraTx, pattern: &AddressPattern) -> Result<bool, Error> {
    let x = tx
        .outputs()
        .iter()
        .filter_map(|o| o.address().ok())
        .any(|a| pattern.matches(a));

    Ok(x)
}

#[inline]
fn eval_input_address(
    tx: &MultiEraTx,
    ctx: &model::BlockContext,
    pattern: &AddressPattern,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<bool, Error> {
    for input in tx.inputs() {
        let utxo = ctx.find_utxo(&input.output_ref()).apply_policy(policy)?;
        if let Some(utxo) = utxo {
            if let Some(addr) = utxo.address().ok() {
                if pattern.matches(addr) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

#[inline]
fn eval_collateral_address(
    tx: &MultiEraTx,
    ctx: &model::BlockContext,
    pattern: &AddressPattern,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<bool, Error> {
    for input in tx.collateral() {
        let utxo = ctx.find_utxo(&input.output_ref()).apply_policy(policy)?;
        if let Some(utxo) = utxo {
            if let Some(addr) = utxo.address().ok() {
                if pattern.matches(addr) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

#[inline]
fn eval_withdrawal_address(tx: &MultiEraTx, pattern: &AddressPattern) -> Result<bool, Error> {
    let x = tx
        .withdrawals()
        .collect::<Vec<_>>()
        .iter()
        .filter_map(|(b, _)| Address::from_bytes(b).ok())
        .any(|a| pattern.matches(a));

    Ok(x)
}

fn eval_address(
    tx: &MultiEraTx,
    ctx: &model::BlockContext,
    pattern: &AddressPattern,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<bool, Error> {
    if eval_output_address(tx, pattern)? {
        return Ok(true);
    }

    if eval_input_address(tx, ctx, pattern, policy)? {
        return Ok(true);
    }

    if eval_withdrawal_address(tx, pattern)? {
        return Ok(true);
    }

    if eval_collateral_address(tx, ctx, pattern, policy)? {
        return Ok(true);
    }

    Ok(false)
}

fn eval_block(block: &MultiEraBlock, pattern: &BlockPattern) -> Result<bool, Error> {
    if let Some(x) = pattern.slot_after {
        return Ok(block.slot() > x);
    }

    if let Some(x) = pattern.slot_before {
        return Ok(block.slot() < x);
    }

    Ok(false)
}

fn eval_transaction(tx: &MultiEraTx, pattern: &TransactionPattern) -> Result<bool, Error> {
    if let Some(b) = pattern.is_valid {
        return Ok(tx.is_valid() == b);
    }

    Ok(false)
}

#[inline]
fn eval_any_of(
    predicates: &[Predicate],
    block: &MultiEraBlock,
    tx: &MultiEraTx,
    ctx: &model::BlockContext,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<bool, Error> {
    for p in predicates.iter() {
        if eval_predicate(p, block, tx, ctx, policy)? {
            return Ok(true);
        }
    }

    Ok(false)
}

#[inline]
fn eval_all_of(
    predicates: &[Predicate],
    block: &MultiEraBlock,
    tx: &MultiEraTx,
    ctx: &model::BlockContext,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<bool, Error> {
    for p in predicates.iter() {
        if !eval_predicate(p, block, tx, ctx, policy)? {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn eval_predicate(
    predicate: &Predicate,
    block: &MultiEraBlock,
    tx: &MultiEraTx,
    ctx: &model::BlockContext,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<bool, Error> {
    match predicate {
        Predicate::Not(x) => eval_predicate(x, block, tx, ctx, policy).map(|x| !x),
        Predicate::AnyOf(x) => eval_any_of(x, block, tx, ctx, policy),
        Predicate::AllOf(x) => eval_all_of(x, block, tx, ctx, policy),
        Predicate::OutputAddress(x) => eval_output_address(tx, x),
        Predicate::InputAddress(x) => eval_input_address(tx, ctx, x, policy),
        Predicate::WithdrawalAddress(x) => eval_withdrawal_address(tx, x),
        Predicate::CollateralAddress(x) => eval_collateral_address(tx, ctx, x, policy),
        Predicate::Address(x) => eval_address(tx, ctx, x, policy),
        Predicate::Block(x) => eval_block(block, x),
        Predicate::Transaction(x) => eval_transaction(tx, x),
    }
}

#[cfg(test)]
mod tests {
    use pallas::ledger::traverse::MultiEraBlock;

    use crate::{
        crosscut::policies::{ErrorAction, RuntimePolicy},
        framework::model::BlockContext,
    };

    use super::{eval_predicate, AddressPattern, Predicate};

    fn test_predicate_in_block(predicate: &Predicate, expected_txs: &[usize]) {
        let cbor = include_str!("../../assets/test.block");
        let bytes = hex::decode(cbor).unwrap();
        let block = MultiEraBlock::decode(&bytes).unwrap();
        let ctx = BlockContext::default();
        let policy = RuntimePolicy {
            missing_data: Some(ErrorAction::Skip),
            ..Default::default()
        };

        let idxs: Vec<_> = block
            .txs()
            .iter()
            .enumerate()
            .filter(|(_, tx)| eval_predicate(predicate, &block, tx, &ctx, &policy).unwrap())
            .map(|(idx, _)| idx)
            .collect();

        assert_eq!(idxs, expected_txs);
    }

    #[test]
    fn output_to_exact_address() {
        let x = Predicate::OutputAddress(AddressPattern {
            exact_bech32: Some("addr1q8fukvydr8m5y3gztte3d4tnw0v5myvshusmu45phf20h395kqnygcykgjy42m29tksmwnd0js0z8p3swm5ntryhfu8sg7835c".into()),
            ..Default::default()
        });

        test_predicate_in_block(&x, &[0]);
    }

    #[test]
    fn exact_address() {
        let x = Predicate::Address(AddressPattern {
            exact_bech32: Some("addr1q8fukvydr8m5y3gztte3d4tnw0v5myvshusmu45phf20h395kqnygcykgjy42m29tksmwnd0js0z8p3swm5ntryhfu8sg7835c".into()),
            ..Default::default()
        });

        test_predicate_in_block(&x, &[0]);
    }

    #[test]
    fn output_to_script_address() {
        let x = Predicate::OutputAddress(AddressPattern {
            is_script: Some(true),
            ..Default::default()
        });

        test_predicate_in_block(&x, &[]);
    }

    #[test]
    fn any_of() {
        let a = Predicate::OutputAddress(AddressPattern {
            exact_bech32: Some("addr1q8fukvydr8m5y3gztte3d4tnw0v5myvshusmu45phf20h395kqnygcykgjy42m29tksmwnd0js0z8p3swm5ntryhfu8sg7835c".into()),
            ..Default::default()
        });

        let b = Predicate::OutputAddress(AddressPattern {
            is_script: Some(true),
            ..Default::default()
        });

        let x = Predicate::AnyOf(vec![a, b]);

        test_predicate_in_block(&x, &[0]);
    }
}
