macro_rules! filter_matches {
    ($reducer:ident, $block:expr, $tx:expr, $ctx:expr) => {
        match &$reducer.config.filter {
            Some(x) => crosscut::filters::eval_predicate(x, $block, $tx, $ctx, &$reducer.policy)
                .or_panic()?,
            // if we don't have a filter, everything goes through
            None => true,
        }
    };
}

macro_rules! filter_matches_block {
    ($reducer:ident, $block:expr, $ctx:expr) => {
        match &$reducer.config.filter {
            Some(x) => {
                // match the block if any of the contained txs satisfy the predicates
                let mut ret = false;
                
                for tx in $block.txs().into_iter() {
                    ret |= crosscut::filters::eval_predicate(x, $block, &tx, $ctx, &$reducer.policy).or_panic()?;
                }
                
                ret
            }
            // if we don't have a filter, everything goes through
            None => true,
        }
    };
}

pub(crate) use filter_matches;
pub(crate) use filter_matches_block;
