// Form → ProposedTrade adapter. Extracted from new_trade.js so node --test
// can verify the shape conversion (execution side → trade side mapping,
// stop_loss parsing, defaults) without a DOM stub.

/**
 * Map an execution-side keyword (buy/sell/short/cover) to the TradeSide
 * the engine expects (long/short).
 *
 * `buy` and `sell` are long-side operations (sell closes a long position).
 * `short` and `cover` are short-side. The Risk Gate evaluates the OPENING
 * trade direction, so this mapping is "what direction does this leg
 * belong to" not "what direction is this leg."
 */
export function executionSideToTradeSide(execSide) {
    return execSide === 'buy' || execSide === 'sell' ? 'long' : 'short';
}

/**
 * Build the ProposedTrade payload from a flat form-data object.
 * Mirrors the inline construction in new_trade.js so both stay in sync.
 */
export function buildProposedTrade(form) {
    return {
        symbol: form.symbol,
        side:   executionSideToTradeSide(form.side),
        qty:    form.qty,
        entry_price: form.price,
        stop_loss: form.stop_loss != null && form.stop_loss !== ''
            ? Number(form.stop_loss) : null,
        asset_class: form.asset_class || 'stock',
        multiplier:  form.multiplier ?? 1,
        tick_size:   null,
        tick_value:  null,
        has_attached_plan: !!form.has_attached_plan,
    };
}

/**
 * Partition a GateDecision's violations array into blocks vs warnings —
 * what the new-trade UI uses to decide between a hard alert and a soft
 * confirm-then-proceed.
 */
export function splitViolations(decision) {
    if (!decision || !Array.isArray(decision.violations)) {
        return { blocks: [], warnings: [] };
    }
    const blocks = decision.violations.filter(v => v.severity === 'block');
    const warnings = decision.violations.filter(v => v.severity === 'warning');
    return { blocks, warnings };
}
