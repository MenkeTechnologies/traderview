// Canonical ISO 4217 currency list used by every "base currency" /
// account currency picker in the app. Replaces the loose 3-char text
// inputs that previously allowed "uds" / lowercase / typos.
//
// Ordered: most-used at the top, then alphabetical. Add codes here
// when a new fiat (or crypto) needs to be supported in account base
// currency, expense base currency, etc.

export const CURRENCIES = [
    { code: 'USD', name: 'US Dollar' },
    { code: 'EUR', name: 'Euro' },
    { code: 'GBP', name: 'British Pound' },
    { code: 'JPY', name: 'Japanese Yen' },
    { code: 'CHF', name: 'Swiss Franc' },
    { code: 'CAD', name: 'Canadian Dollar' },
    { code: 'AUD', name: 'Australian Dollar' },
    { code: 'NZD', name: 'New Zealand Dollar' },
    // ── Alphabetical from here ─────────────────────────────────────
    { code: 'AED', name: 'UAE Dirham' },
    { code: 'ARS', name: 'Argentine Peso' },
    { code: 'BRL', name: 'Brazilian Real' },
    { code: 'CLP', name: 'Chilean Peso' },
    { code: 'CNY', name: 'Chinese Yuan' },
    { code: 'COP', name: 'Colombian Peso' },
    { code: 'CZK', name: 'Czech Koruna' },
    { code: 'DKK', name: 'Danish Krone' },
    { code: 'HKD', name: 'Hong Kong Dollar' },
    { code: 'HUF', name: 'Hungarian Forint' },
    { code: 'IDR', name: 'Indonesian Rupiah' },
    { code: 'ILS', name: 'Israeli Shekel' },
    { code: 'INR', name: 'Indian Rupee' },
    { code: 'KRW', name: 'South Korean Won' },
    { code: 'MXN', name: 'Mexican Peso' },
    { code: 'MYR', name: 'Malaysian Ringgit' },
    { code: 'NOK', name: 'Norwegian Krone' },
    { code: 'PEN', name: 'Peruvian Sol' },
    { code: 'PHP', name: 'Philippine Peso' },
    { code: 'PLN', name: 'Polish Zloty' },
    { code: 'RON', name: 'Romanian Leu' },
    { code: 'RUB', name: 'Russian Ruble' },
    { code: 'SAR', name: 'Saudi Riyal' },
    { code: 'SEK', name: 'Swedish Krona' },
    { code: 'SGD', name: 'Singapore Dollar' },
    { code: 'THB', name: 'Thai Baht' },
    { code: 'TRY', name: 'Turkish Lira' },
    { code: 'TWD', name: 'Taiwan Dollar' },
    { code: 'VND', name: 'Vietnamese Dong' },
    { code: 'ZAR', name: 'South African Rand' },
    // ── Crypto (USD-pegged + majors) — useful for crypto exchanges ─
    { code: 'BTC', name: 'Bitcoin' },
    { code: 'ETH', name: 'Ether' },
    { code: 'USDC', name: 'USD Coin' },
    { code: 'USDT', name: 'Tether' },
];

const CURRENCY_CODES = new Set(CURRENCIES.map((c) => c.code));

/** True if `code` is a known currency. Case-insensitive. */
export function isKnownCurrency(code) {
    return CURRENCY_CODES.has(String(code || '').toUpperCase());
}

/**
 * Build `<option>` markup for a currency `<select>`. Pass the
 * currently-selected code so it gets pre-selected; defaults to 'USD'.
 * Unknown codes get appended at the end of the list as a one-off
 * preserve-as-typed option (so editing an older account with a code we
 * don't ship doesn't silently drop it on save).
 */
export function currencyOptions(selected = 'USD') {
    const sel = String(selected || 'USD').toUpperCase();
    const baseHtml = CURRENCIES.map((c) =>
        `<option value="${c.code}"${c.code === sel ? ' selected' : ''}>${c.code} — ${c.name}</option>`,
    ).join('');
    if (!CURRENCY_CODES.has(sel)) {
        return `<option value="${escAttr(sel)}" selected>${escAttr(sel)}</option>${baseHtml}`;
    }
    return baseHtml;
}

function escAttr(s) {
    return String(s).replace(/[&<>"]/g, (c) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;',
    }[c]));
}
