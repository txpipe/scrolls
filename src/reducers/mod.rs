use gasket::messaging::{InputPort, OutputPort};

use crate::{bootstrap, crosscut, model};

pub mod point_by_tx;
pub mod pool_by_stake;
pub mod utxo_by_address;
pub mod total_transactions_count;
pub mod transactions_count_by_epoch;

pub trait Pluggable {
    fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::ChainSyncCommandEx>;
    fn borrow_output_port(&mut self) -> &'_ mut OutputPort<model::CRDTCommand>;
    fn spawn(self, pipeline: &mut bootstrap::Pipeline);
}

pub enum Plugin {
    UtxoByAddress(utxo_by_address::Worker),
    PointByTx(point_by_tx::Worker),
    PoolByStake(pool_by_stake::Worker),
    TotalTransactionsCount(total_transactions_count::Worker),
    TransactionsCountByEpoch(transactions_count_by_epoch::Worker),
}

impl Plugin {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::ChainSyncCommandEx> {
        match self {
            Plugin::UtxoByAddress(x) => x.borrow_input_port(),
            Plugin::PointByTx(x) => x.borrow_input_port(),
            Plugin::PoolByStake(x) => x.borrow_input_port(),
            Plugin::TotalTransactionsCount(x) => x.borrow_input_port(),
            Plugin::TransactionsCountByEpoch(x) => x.borrow_input_port(),
        }
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<model::CRDTCommand> {
        match self {
            Plugin::UtxoByAddress(x) => x.borrow_output_port(),
            Plugin::PointByTx(x) => x.borrow_output_port(),
            Plugin::PoolByStake(x) => x.borrow_output_port(),
            Plugin::TotalTransactionsCount(x) => x.borrow_output_port(),
            Plugin::TransactionsCountByEpoch(x) => x.borrow_output_port(),
        }
    }

    pub fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Plugin::UtxoByAddress(x) => x.spawn(pipeline),
            Plugin::PointByTx(x) => x.spawn(pipeline),
            Plugin::PoolByStake(x) => x.spawn(pipeline),
            Plugin::TotalTransactionsCount(x) => x.spawn(pipeline),
            Plugin::TransactionsCountByEpoch(x) => x.spawn(pipeline),
        }
    }

}

pub trait IntoPlugin {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
    ) -> Plugin;
}
