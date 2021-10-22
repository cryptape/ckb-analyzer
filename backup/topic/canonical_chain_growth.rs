///
/// ## Difference with `Topic::canonical_chain_state`
///
/// * `Topic::canonical_chain_state` aims the chain state; the chain state history is the same over the
/// network nodes;
///
/// * This topic aims the chain growth history on a specific node. The chain growth history is
/// different over nodes.
///
/// ## How it works
///
/// * Ckb node record its growth events into log-file;
/// * Watch log-file and report the growth events.
