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

pub(crate) use filter_matches;
