use gasket::error::AsWorkError;
use pallas::ledger::{
    addresses::Address,
    traverse::{MultiEraBlock, MultiEraTx},
};

use crate::{model::BlockContext, prelude::AppliesPolicy};

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

pub struct AssetPattern {
    policy: Option<String>,
    name: Option<String>,
    subject: Option<String>,
}

pub struct BlockPattern {
    slot_before: u64,
    slot_after: u64,
}

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

fn payment_to_matches(tx: &MultiEraTx, pattern: &AddressPattern) -> bool {
    tx.outputs()
        .iter()
        .filter_map(|o| o.address().ok())
        .any(|a| pattern.matches(a))
}

fn payment_from_matches(tx: &MultiEraTx, ctx: &BlockContext, pattern: &AddressPattern) -> bool {
    tx.inputs()
        .iter()
        .map(|inp| ctx.find_utxo(&inp.output_ref()))
        .filter_map(|inp| inp.apply_policy(policy).or_panic().unwrap())
        .filter_map(|inp| inp.address().ok())
        .any(|addr| pattern.matches(addr))
}

impl Predicate {
    fn matches(&self, block: &MultiEraBlock, tx: &MultiEraTx, ctx: &BlockContext) -> bool {
        match self {
            Predicate::Not(x) => !x.matches(block, tx, ctx),
            Predicate::AnyOf(x) => x.iter().any(|c| c.matches(block, tx, ctx)),
            Predicate::AllOf(x) => x.iter().all(|c| c.matches(block, tx, ctx)),
            Predicate::PaymentTo(x) => payment_to_matches(tx, x),
            Predicate::PaymentFrom(x) => payment_from_matches(tx, ctx, x),
            Predicate::WithdrawalTo(_) => todo!(),
            Predicate::InputAsset(_) => todo!(),
            Predicate::OutputAsset(_) => todo!(),
            Predicate::Block(_) => todo!(),
        }
    }
}
