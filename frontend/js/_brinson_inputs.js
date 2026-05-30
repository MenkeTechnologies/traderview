// Brinson (1986) performance attribution helpers.
//
// Backend body: { inputs: [{sector, portfolio_weight, benchmark_weight,
//   portfolio_return, benchmark_return}, ...] }
// Returns: { per_sector: [{sector, allocation_effect, selection_effect,
//   interaction_effect}], total_allocation, total_selection,
//   total_interaction, portfolio_total_return, benchmark_total_return,
//   total_active_return } | null
//
// Identity:
//   total_active = portfolio_total_return − benchmark_total_return
//                = Σ (A_i + S_i + I_i)
// where
//   A_i = (w_p − w_b) · (r_b − r_b_total)         allocation
//   S_i = w_b · (r_p − r_b)                       selection
//   I_i = (w_p − w_b) · (r_p − r_b)               interaction

import { t } from './i18n.js';

export const DEFAULT_INPUTS = { inputs: [] };

export function validateInputs(input) {
    if (!Array.isArray(input.inputs))                                  return t('view.brinson.validate.inputs_array');
    if (input.inputs.length === 0)                                      return t('view.brinson.validate.inputs_empty');
    for (let i = 0; i < input.inputs.length; i++) {
        const s = input.inputs[i];
        if (!s || typeof s !== 'object')                               return t('view.brinson.validate.row_object', { i });
        if (typeof s.sector !== 'string' || s.sector.length === 0)     return t('view.brinson.validate.sector', { i });
        if (!Number.isFinite(s.portfolio_weight) || s.portfolio_weight < 0)
                                                                        return t('view.brinson.validate.port_weight', { i });
        if (!Number.isFinite(s.benchmark_weight) || s.benchmark_weight < 0)
                                                                        return t('view.brinson.validate.bench_weight', { i });
        if (!Number.isFinite(s.portfolio_return))                      return t('view.brinson.validate.port_return', { i });
        if (!Number.isFinite(s.benchmark_return))                      return t('view.brinson.validate.bench_return', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        inputs: input.inputs.map(s => ({
            sector:           s.sector,
            portfolio_weight: s.portfolio_weight,
            benchmark_weight: s.benchmark_weight,
            portfolio_return: s.portfolio_return,
            benchmark_return: s.benchmark_return,
        })),
    };
}

// Pure-JS mirror of crates/traderview-core/src/brinson_attribution.rs::analyze.
export function localAnalyze(inputs) {
    if (!Array.isArray(inputs) || inputs.length === 0) return null;
    for (const s of inputs) {
        if (!Number.isFinite(s.portfolio_weight)) return null;
        if (!Number.isFinite(s.benchmark_weight)) return null;
        if (!Number.isFinite(s.portfolio_return)) return null;
        if (!Number.isFinite(s.benchmark_return)) return null;
        if (s.portfolio_weight < 0 || s.benchmark_weight < 0) return null;
    }
    let benchmark_total = 0, portfolio_total = 0;
    for (const s of inputs) {
        benchmark_total += s.benchmark_weight * s.benchmark_return;
        portfolio_total += s.portfolio_weight * s.portfolio_return;
    }
    let total_a = 0, total_s = 0, total_i = 0;
    const per_sector = [];
    for (const s of inputs) {
        const dw = s.portfolio_weight - s.benchmark_weight;
        const dr = s.portfolio_return - s.benchmark_return;
        const allocation = dw * (s.benchmark_return - benchmark_total);
        const selection = s.benchmark_weight * dr;
        const interaction = dw * dr;
        total_a += allocation;
        total_s += selection;
        total_i += interaction;
        per_sector.push({
            sector: s.sector,
            allocation_effect: allocation,
            selection_effect: selection,
            interaction_effect: interaction,
        });
    }
    return {
        per_sector,
        total_allocation:        total_a,
        total_selection:         total_s,
        total_interaction:       total_i,
        portfolio_total_return:  portfolio_total,
        benchmark_total_return:  benchmark_total,
        total_active_return:     portfolio_total - benchmark_total,
    };
}

// Parse "sector portfolio_w benchmark_w portfolio_r benchmark_r" per line.
// Weights/returns accept decimals (0.30) or pct-suffix ("30%").
export function parseInputsBlob(blob) {
    const out = { inputs: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 5) {
            out.errors.push({ line_no: i + 1, message: 'expected 5 tokens (sector port_w bench_w port_r bench_r)' });
            continue;
        }
        const sector = toks[0];
        const pw = pctOrDec(toks[1]);
        const bw = pctOrDec(toks[2]);
        const pr = pctOrDec(toks[3]);
        const br = pctOrDec(toks[4]);
        if (!Number.isFinite(pw) || pw < 0) {
            out.errors.push({ line_no: i + 1, message: 'portfolio_weight must be ≥ 0 finite' });
            continue;
        }
        if (!Number.isFinite(bw) || bw < 0) {
            out.errors.push({ line_no: i + 1, message: 'benchmark_weight must be ≥ 0 finite' });
            continue;
        }
        if (!Number.isFinite(pr) || !Number.isFinite(br)) {
            out.errors.push({ line_no: i + 1, message: 'returns must be finite' });
            continue;
        }
        out.inputs.push({
            sector, portfolio_weight: pw, benchmark_weight: bw,
            portfolio_return: pr, benchmark_return: br,
        });
    }
    return out;
}

function pctOrDec(tok) {
    if (tok.endsWith('%')) {
        const v = Number(tok.slice(0, -1));
        return Number.isFinite(v) ? v / 100 : NaN;
    }
    return Number(tok);
}

export function inputsToBlob(inputs) {
    return inputs.map(s =>
        `${s.sector} ${s.portfolio_weight} ${s.benchmark_weight} ${s.portfolio_return} ${s.benchmark_return}`
    ).join('\n');
}

// Verdict on total active return.
export function activeBadge(active) {
    if (!Number.isFinite(active)) return { key: 'view.brinson.badge.unknown', cls: '' };
    if (active >= 0.02)            return { key: 'view.brinson.badge.strong_alpha', cls: 'pos' };
    if (active >= 0.005)           return { key: 'view.brinson.badge.alpha',        cls: 'pos' };
    if (active >  -0.005)          return { key: 'view.brinson.badge.flat',         cls: '' };
    if (active >  -0.02)           return { key: 'view.brinson.badge.lagging',      cls: 'neg' };
    return { key: 'view.brinson.badge.deep_lag', cls: 'neg' };
}

// Which effect dominates the active return?
export function driverBadge(report) {
    if (!report) return { key: 'view.brinson.driver.unknown', cls: '' };
    const a = Math.abs(report.total_allocation);
    const s = Math.abs(report.total_selection);
    const i = Math.abs(report.total_interaction);
    if (a + s + i === 0) return { key: 'view.brinson.driver.none', cls: '' };
    if (a >= s && a >= i) return { key: 'view.brinson.driver.allocation',  cls: '' };
    if (s >= a && s >= i) return { key: 'view.brinson.driver.selection',   cls: '' };
    return { key: 'view.brinson.driver.interaction', cls: '' };
}

// Per-sector enrichment for the table view.
export function enrichSector(s, sectorEffect) {
    const total = sectorEffect.allocation_effect + sectorEffect.selection_effect + sectorEffect.interaction_effect;
    return {
        ...sectorEffect,
        portfolio_weight: s.portfolio_weight,
        benchmark_weight: s.benchmark_weight,
        portfolio_return: s.portfolio_return,
        benchmark_return: s.benchmark_return,
        weight_diff:      s.portfolio_weight - s.benchmark_weight,
        return_diff:      s.portfolio_return - s.benchmark_return,
        total_effect:     total,
    };
}

// Demos.
export function makeDemoInput(kind = 'mixed') {
    switch (kind) {
        case 'identical': {
            return { inputs: [
                row('Tech',   0.4, 0.4, 0.05, 0.05),
                row('Energy', 0.3, 0.3, -0.02, -0.02),
                row('Health', 0.3, 0.3, 0.01, 0.01),
            ]};
        }
        case 'allocation-win': {
            // Overweight a sector that beats the benchmark mean.
            return { inputs: [
                row('Tech',  0.6, 0.4, 0.05, 0.05),    // winner overweight
                row('Other', 0.4, 0.6, 0.01, 0.01),
            ]};
        }
        case 'selection-win': {
            // Same weights, better stock picks within sectors.
            return { inputs: [
                row('Tech',  0.4, 0.4, 0.10, 0.05),
                row('Other', 0.6, 0.6, 0.02, 0.02),
            ]};
        }
        case 'mixed': {
            // Classic 4-sector mix.
            return { inputs: [
                row('Tech',   0.30, 0.20,  0.12, 0.08),
                row('Energy', 0.15, 0.25, -0.03, 0.01),
                row('Health', 0.25, 0.20,  0.05, 0.04),
                row('Fin',    0.30, 0.35,  0.02, 0.03),
            ]};
        }
        case 'losing-overweight': {
            return { inputs: [
                row('Energy', 0.40, 0.20, -0.05, -0.04),
                row('Tech',   0.20, 0.40,  0.08,  0.07),
                row('Other',  0.40, 0.40,  0.01,  0.01),
            ]};
        }
        case 'cash-heavy': {
            // 25% cash (zero benchmark weight, zero return).
            return { inputs: [
                row('Cash',   0.25, 0.0,  0.00, 0.00),
                row('Equity', 0.75, 1.0,  0.10, 0.12),
            ]};
        }
        case 'sector-bet': {
            return { inputs: [
                row('AI',     0.40, 0.05,  0.25, 0.20),
                row('Banks',  0.10, 0.20, -0.02, -0.01),
                row('Health', 0.20, 0.20,  0.05, 0.04),
                row('Other',  0.30, 0.55,  0.03, 0.02),
            ]};
        }
        case 'all-effects': {
            // Allocation, selection AND interaction all non-zero.
            return { inputs: [
                row('Tech',  0.6, 0.4, 0.10, 0.05),
                row('Other', 0.4, 0.6, 0.02, 0.02),
            ]};
        }
        default: return makeDemoInput('mixed');
    }
}

function row(sector, pw, bw, pr, br) {
    return { sector, portfolio_weight: pw, benchmark_weight: bw,
             portfolio_return: pr, benchmark_return: br };
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtPctSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtBps(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 10_000).toFixed(d) + ' bps';
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
