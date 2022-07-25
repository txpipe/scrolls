use pallas::ledger::{
    addresses::Address,
    traverse::{MultiEraBlock, MultiEraTx},
};
use serde::Deserialize;

use crate::prelude::*;
use crate::{crosscut, model};

#[derive(Deserialize, Clone)]
pub struct AddressPattern {
    pub exact: Option<String>,
    pub payment: Option<String>,
    pub stake: Option<String>,
    pub is_script: Option<bool>,
}

impl AddressPattern {
    pub fn matches(&self, addr: Address) -> bool {
        if let Some(x) = &self.exact {
            if addr.to_string().eq(x) {
                return true;
            }
        }

        if let Some(_) = &self.payment {
            todo!();
        }

        if let Some(_) = &self.stake {
            todo!();
        }

        if let Some(x) = &self.is_script {
            return addr.has_script() == *x;
        }

        false
    }
}

#[derive(Deserialize, Clone)]
pub struct AssetPattern {
    pub policy: Option<String>,
    pub name: Option<String>,
    pub subject: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct BlockPattern {
    pub slot_before: u64,
    pub slot_after: u64,
}

#[derive(Deserialize, Clone)]
pub enum Predicate {
    AllOf(Vec<Predicate>),
    AnyOf(Vec<Predicate>),
    Not(Box<Predicate>),
    PaymentTo(AddressPattern),
    PaymentFrom(AddressPattern),
    WithdrawalTo(AddressPattern),
    InputAsset(AssetPattern),
    OutputAsset(AssetPattern),
    Block(BlockPattern),
}

impl Predicate {
    pub fn and(&self, other: &Self) -> Self {
        Predicate::AllOf(vec![self.clone(), other.clone()])
    }
}

#[inline]
fn eval_payment_to(tx: &MultiEraTx, pattern: &AddressPattern) -> Result<bool, crate::Error> {
    let x = tx
        .outputs()
        .iter()
        .filter_map(|o| o.address().ok())
        .any(|a| pattern.matches(a));

    Ok(x)
}

#[inline]
fn eval_payment_from(
    tx: &MultiEraTx,
    ctx: &model::BlockContext,
    pattern: &AddressPattern,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<bool, crate::Error> {
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
fn eval_any_of(
    predicates: &[Predicate],
    block: &MultiEraBlock,
    tx: &MultiEraTx,
    ctx: &model::BlockContext,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<bool, crate::Error> {
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
) -> Result<bool, crate::Error> {
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
) -> Result<bool, crate::Error> {
    match predicate {
        Predicate::Not(x) => eval_predicate(x, block, tx, ctx, policy).map(|x| !x),
        Predicate::AnyOf(x) => eval_any_of(x, block, tx, ctx, policy),
        Predicate::AllOf(x) => eval_all_of(x, block, tx, ctx, policy),
        Predicate::PaymentTo(x) => eval_payment_to(tx, x),
        Predicate::PaymentFrom(x) => eval_payment_from(tx, ctx, x, policy),
        Predicate::WithdrawalTo(_) => todo!(),
        Predicate::InputAsset(_) => todo!(),
        Predicate::OutputAsset(_) => todo!(),
        Predicate::Block(_) => todo!(),
    }
}
