// Keyboard-shortcut registry — pure helpers shared with vitest.
//
// Shape per shortcut:
//   { id, keys: { key: 'k', meta: true, ctrl: false, shift: false, alt: false },
//     descKey, scope: 'global' | 'palette' | 'editor', actionKey }
//
// `actionKey` is a CustomEvent name dispatched on `window` when the
// shortcut fires (e.g. 'tv:open-palette'). The wiring layer
// (frontend/js/shortcuts.js) translates DOM keydown → registry lookup →
// `window.dispatchEvent(new CustomEvent(actionKey))`.

export const LS_KEY = 'tv-shortcuts-v1';
export const VERSION = 1;

export const DEFAULT_SHORTCUTS = [
    { id: 'command_palette', keys: { key: 'k', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.command_palette', actionKey: 'tv:open-palette' },
    { id: 'help',            keys: { key: '?', meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.help',           actionKey: 'tv:open-help' },
    { id: 'escape',          keys: { key: 'Escape', meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.escape', actionKey: 'tv:escape' },
    { id: 'focus_search',    keys: { key: '/', meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.focus_search',   actionKey: 'tv:focus-search' },
    { id: 'reload',          keys: { key: 'r', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.reload',         actionKey: 'tv:reload' },
    { id: 'toggle_favorite', keys: { key: 'd', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.toggle_favorite', actionKey: 'tv:toggle-favorite' },
    { id: 'open_new_tab',    keys: { key: 'n', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.open_new_tab',    actionKey: 'tv:open-new-tab' },
    { id: 'add_bookmark',    keys: { key: 'b', meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.add_bookmark',    actionKey: 'tv:add-bookmark' },
    { id: 'go_home',         keys: { key: 'h', meta: true,  ctrl: true,  shift: true,  alt: false }, scope: 'global',  descKey: 'shortcut.go_home',         actionKey: 'tv:go-home' },
    // No default keybind for clear_recents — it's destructive enough that a single keypress would be too easy. Surface it only via palette + ctx menu.
    { id: 'clear_recents',   keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.clear_recents',   actionKey: 'tv:clear-recents' },
    { id: 'toggle_theme',    keys: { key: 'l', meta: true,  ctrl: true,  shift: true,  alt: false }, scope: 'global',  descKey: 'shortcut.toggle_theme',    actionKey: 'tv:toggle-theme' },
    { id: 'toggle_crt',      keys: { key: 'c', meta: true,  ctrl: true,  shift: true,  alt: false }, scope: 'global',  descKey: 'shortcut.toggle_crt',      actionKey: 'tv:toggle-crt' },
    { id: 'toggle_neon',     keys: { key: 'g', meta: true,  ctrl: true,  shift: true,  alt: false }, scope: 'global',  descKey: 'shortcut.toggle_neon',     actionKey: 'tv:toggle-neon' },
    { id: 'cycle_locale',    keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.cycle_locale',    actionKey: 'tv:cycle-locale' },
    { id: 'open_settings',   keys: { key: ',',  meta: true,  ctrl: true,  shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.open_settings',   actionKey: 'tv:open-settings' },
    // Context-menu actions surfaced in palette only — no default binding.
    { id: 'nav_back',        keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.nav_back',        actionKey: 'tv:nav-back' },
    { id: 'copy_view_url',   keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.copy_view_url',   actionKey: 'tv:copy-view-url' },
    { id: 'copy_view_id',    keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.copy_view_id',    actionKey: 'tv:copy-view-id' },
    // Text-entry edit actions — palette finds them; activation falls
    // back to document.activeElement when no ctxmenu target was set.
    { id: 'edit_cut',        keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_cut',        actionKey: 'tv:edit-cut' },
    { id: 'edit_copy',       keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_copy',       actionKey: 'tv:edit-copy' },
    { id: 'edit_paste',      keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_paste',      actionKey: 'tv:edit-paste' },
    { id: 'edit_select_all', keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_select_all', actionKey: 'tv:edit-select-all' },
    { id: 'edit_undo',       keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_undo',       actionKey: 'tv:edit-undo' },
    { id: 'edit_redo',       keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global',  descKey: 'shortcut.edit_redo',       actionKey: 'tv:edit-redo' },
    // Symbol-aware quick-nav. No default keybinding — surfaced via the
    // command palette and right-click context menu only. Acts on the
    // current global symbol; toasts "no active symbol" if unset.
    { id: 'copy_symbol',              keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.copy_symbol',              actionKey: 'tv:copy-symbol' },
    { id: 'open_charts_for_symbol',   keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_charts_for_symbol',   actionKey: 'tv:open-charts-for-symbol' },
    { id: 'open_options_for_symbol',  keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_options_for_symbol',  actionKey: 'tv:open-options-for-symbol' },
    { id: 'open_research_for_symbol', keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_research_for_symbol', actionKey: 'tv:open-research-for-symbol' },
    { id: 'open_earnings_for_symbol', keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_earnings_for_symbol', actionKey: 'tv:open-earnings-for-symbol' },
    { id: 'open_news_for_symbol',     keys: { key: null, meta: false, ctrl: false, shift: false, alt: false }, scope: 'global', descKey: 'shortcut.open_news_for_symbol',     actionKey: 'tv:open-news-for-symbol' },
    // View-scoped: fire only when the dispatcher's setScope(view)
    // matches. The trades scope unlocks `n` for new-trade; the
    // dashboard scope unlocks `r` for refresh. Neither triggers in
    // text-entry context (handled by isTextEntryTarget + non-modifier).
    { id: 'trades_new',         keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'trades',    descKey: 'shortcut.trades_new',         actionKey: 'tv:trades-new' },
    { id: 'dashboard_refresh',  keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'dashboard', descKey: 'shortcut.dashboard_refresh',  actionKey: 'tv:dashboard-refresh' },
    { id: 'journal_focus_body', keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'journal',   descKey: 'shortcut.journal_focus_body', actionKey: 'tv:journal-focus-body' },
    { id: 'watchlists_focus_add', keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'watchlists',  descKey: 'shortcut.watchlists_focus_add',  actionKey: 'tv:watchlists-focus-add' },
    { id: 'alert_rules_focus_new', keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'alert-rules', descKey: 'shortcut.alert_rules_focus_new', actionKey: 'tv:alert-rules-focus-new' },
    { id: 'rebalance_compute',       keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'rebalance',   descKey: 'shortcut.rebalance_compute',       actionKey: 'tv:rebalance-compute' },
    { id: 'rebalance_focus_targets', keys: { key: 't', meta: false, ctrl: false, shift: false, alt: false }, scope: 'rebalance',   descKey: 'shortcut.rebalance_focus_targets', actionKey: 'tv:rebalance-focus-targets' },
    { id: 'strategy_alerts_focus_name',   keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'strategy-alerts', descKey: 'shortcut.strategy_alerts_focus_name',   actionKey: 'tv:strategy-alerts-focus-name' },
    { id: 'strategy_alerts_evaluate_now', keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'strategy-alerts', descKey: 'shortcut.strategy_alerts_evaluate_now', actionKey: 'tv:strategy-alerts-evaluate-now' },
    { id: 'accounts_focus_name',          keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'accounts',          descKey: 'shortcut.accounts_focus_name',          actionKey: 'tv:accounts-focus-name' },
    { id: 'accounts_overview_refresh',    keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'accounts-overview', descKey: 'shortcut.accounts_overview_refresh',    actionKey: 'tv:accounts-overview-refresh' },
    { id: 'developer_focus_name',         keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'developer',         descKey: 'shortcut.developer_focus_name',         actionKey: 'tv:developer-focus-name' },
    { id: 'developer_generate',           keys: { key: 'g', meta: false, ctrl: false, shift: false, alt: false }, scope: 'developer',         descKey: 'shortcut.developer_generate',           actionKey: 'tv:developer-generate' },
    { id: 'backtest_run',                 keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'backtest',          descKey: 'shortcut.backtest_run',                 actionKey: 'tv:backtest-run' },
    { id: 'backtest_presets_focus_name',  keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'backtest-presets',  descKey: 'shortcut.backtest_presets_focus_name',  actionKey: 'tv:backtest-presets-focus-name' },
    { id: 'csv_wizard_upload',            keys: { key: 'u', meta: false, ctrl: false, shift: false, alt: false }, scope: 'csv-wizard',        descKey: 'shortcut.csv_wizard_upload',            actionKey: 'tv:csv-wizard-upload' },
    { id: 'boards_focus_name',            keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'boards',            descKey: 'shortcut.boards_focus_name',            actionKey: 'tv:boards-focus-name' },
    { id: 'import_pick_file',             keys: { key: 'p', meta: false, ctrl: false, shift: false, alt: false }, scope: 'import',            descKey: 'shortcut.import_pick_file',             actionKey: 'tv:import-pick-file' },
    { id: 'import_upload',                keys: { key: 'u', meta: false, ctrl: false, shift: false, alt: false }, scope: 'import',            descKey: 'shortcut.import_upload',                actionKey: 'tv:import-upload' },
    { id: 'ai_save',                      keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'ai',                descKey: 'shortcut.ai_save',                      actionKey: 'tv:ai-save' },
    { id: 'community_focus_title',        keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'community',         descKey: 'shortcut.community_focus_title',        actionKey: 'tv:community-focus-title' },
    { id: 'discipline_refresh',           keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'discipline',        descKey: 'shortcut.discipline_refresh',           actionKey: 'tv:discipline-refresh' },
    { id: 'goals_focus_name',             keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'goals',             descKey: 'shortcut.goals_focus_name',             actionKey: 'tv:goals-focus-name' },
    { id: 'journal_save',                 keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'journal',           descKey: 'shortcut.journal_save',                 actionKey: 'tv:journal-save' },
    { id: 'hotkeys_focus_name',           keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'hotkeys',           descKey: 'shortcut.hotkeys_focus_name',           actionKey: 'tv:hotkeys-focus-name' },
    { id: 'hotkeys_capture',              keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'hotkeys',           descKey: 'shortcut.hotkeys_capture',              actionKey: 'tv:hotkeys-capture' },
    { id: 'paper_submit',                 keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'paper',             descKey: 'shortcut.paper_submit',                 actionKey: 'tv:paper-submit' },
    { id: 'screener_run',                 keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'screener',          descKey: 'shortcut.screener_run',                 actionKey: 'tv:screener-run' },
    { id: 'dashboards_focus_new',         keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'dashboards',        descKey: 'shortcut.dashboards_focus_new',         actionKey: 'tv:dashboards-focus-new' },
    { id: 'dashboards_toggle_edit',       keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'dashboards',        descKey: 'shortcut.dashboards_toggle_edit',       actionKey: 'tv:dashboards-toggle-edit' },
    { id: 'new_trade_add',                keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'new-trade',         descKey: 'shortcut.new_trade_add',                actionKey: 'tv:new-trade-add' },
    { id: 'research_action',              keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'research',          descKey: 'shortcut.research_action',              actionKey: 'tv:research-action' },
    { id: 'economy_load',                 keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'economy',           descKey: 'shortcut.economy_load',                 actionKey: 'tv:economy-load' },
    { id: 'earnings_cal_refresh',         keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'earnings-cal',      descKey: 'shortcut.earnings_cal_refresh',         actionKey: 'tv:earnings-cal-refresh' },
    { id: 'earnings_cal_poll',            keys: { key: 'p', meta: false, ctrl: false, shift: false, alt: false }, scope: 'earnings-cal',      descKey: 'shortcut.earnings_cal_poll',            actionKey: 'tv:earnings-cal-poll' },
    { id: 'monte_carlo_run',              keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'monte-carlo',       descKey: 'shortcut.monte_carlo_run',              actionKey: 'tv:monte-carlo-run' },
    { id: 'kelly_compute_static',         keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'kelly',             descKey: 'shortcut.kelly_compute_static',         actionKey: 'tv:kelly-compute-static' },
    { id: 'kelly_compute_dynamic',        keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'kelly',             descKey: 'shortcut.kelly_compute_dynamic',        actionKey: 'tv:kelly-compute-dynamic' },
    { id: 'risk_save',                    keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'risk',              descKey: 'shortcut.risk_save',                    actionKey: 'tv:risk-save' },
    { id: 'darkpool_rank',                keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'darkpool',          descKey: 'shortcut.darkpool_rank',                actionKey: 'tv:darkpool-rank' },
    { id: 'var_calculator_compute',       keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'var-calculator',    descKey: 'shortcut.var_calculator_compute',       actionKey: 'tv:var-calculator-compute' },
    { id: 'portfolio_allocator_run',      keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'portfolio-allocator', descKey: 'shortcut.portfolio_allocator_run',    actionKey: 'tv:portfolio-allocator-run' },
    { id: 'live_scanner_connect',         keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'live-scanner',      descKey: 'shortcut.live_scanner_connect',         actionKey: 'tv:live-scanner-connect' },
    { id: 'live_scanner_toggle_voice',    keys: { key: 'v', meta: false, ctrl: false, shift: false, alt: false }, scope: 'live-scanner',      descKey: 'shortcut.live_scanner_toggle_voice',    actionKey: 'tv:live-scanner-toggle-voice' },
    { id: 'replay_refresh',               keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'replay',            descKey: 'shortcut.replay_refresh',               actionKey: 'tv:replay-refresh' },
    { id: 'top_signals_refresh',          keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'top-signals',       descKey: 'shortcut.top_signals_refresh',          actionKey: 'tv:top-signals-refresh' },
    { id: 'pair_trade_analyze',           keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'pair-trade-calc',   descKey: 'shortcut.pair_trade_analyze',           actionKey: 'tv:pair-trade-analyze' },
    { id: 'vol_smile_fit',                keys: { key: 'f', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vol-smile',         descKey: 'shortcut.vol_smile_fit',                actionKey: 'tv:vol-smile-fit' },
    { id: 'option_payoff_recalc',         keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'option-payoff',     descKey: 'shortcut.option_payoff_recalc',         actionKey: 'tv:option-payoff-recalc' },
    { id: 'series_smoother_run',          keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'series-smoother',   descKey: 'shortcut.series_smoother_run',          actionKey: 'tv:series-smoother-run' },
    { id: 'pattern_discovery_run',        keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'pattern-discovery', descKey: 'shortcut.pattern_discovery_run',        actionKey: 'tv:pattern-discovery-run' },
    { id: 'execution_scheduler_run',      keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'execution-scheduler', descKey: 'shortcut.execution_scheduler_run',    actionKey: 'tv:execution-scheduler-run' },
    { id: 'regime_detector_run',          keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'regime-detector',   descKey: 'shortcut.regime_detector_run',          actionKey: 'tv:regime-detector-run' },
    { id: 'american_option_price',        keys: { key: 'p', meta: false, ctrl: false, shift: false, alt: false }, scope: 'american-option',   descKey: 'shortcut.american_option_price',        actionKey: 'tv:american-option-price' },
    { id: 'fx_option_price',              keys: { key: 'p', meta: false, ctrl: false, shift: false, alt: false }, scope: 'fx-option',         descKey: 'shortcut.fx_option_price',              actionKey: 'tv:fx-option-price' },
    { id: 'greeks_profile_compute',       keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'greeks-profile',    descKey: 'shortcut.greeks_profile_compute',       actionKey: 'tv:greeks-profile-compute' },
    { id: 'iv_solver_solve',              keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'iv-solver',         descKey: 'shortcut.iv_solver_solve',              actionKey: 'tv:iv-solver-solve' },
    { id: 'kalman_beta_run',              keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'kalman-beta',       descKey: 'shortcut.kalman_beta_run',              actionKey: 'tv:kalman-beta-run' },
    { id: 'optimal_f_compute',            keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'optimal-f',         descKey: 'shortcut.optimal_f_compute',            actionKey: 'tv:optimal-f-compute' },
    { id: 'dtw_warp',                     keys: { key: 'w', meta: false, ctrl: false, shift: false, alt: false }, scope: 'dtw',               descKey: 'shortcut.dtw_warp',                     actionKey: 'tv:dtw-warp' },
    { id: 'hurst_estimate',               keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'hurst',             descKey: 'shortcut.hurst_estimate',               actionKey: 'tv:hurst-estimate' },
    { id: 'bocpd_detect',                 keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'bocpd',             descKey: 'shortcut.bocpd_detect',                 actionKey: 'tv:bocpd-detect' },
    { id: 'vasicek_simulate',             keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vasicek',           descKey: 'shortcut.vasicek_simulate',             actionKey: 'tv:vasicek-simulate' },
    { id: 'microprice_compute',           keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'microprice',        descKey: 'shortcut.microprice_compute',           actionKey: 'tv:microprice-compute' },
    { id: 'vpin_compute',                 keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vpin',              descKey: 'shortcut.vpin_compute',                 actionKey: 'tv:vpin-compute' },
    { id: 'vpin_demo',                    keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vpin',              descKey: 'shortcut.vpin_demo',                    actionKey: 'tv:vpin-demo' },
    { id: 'deflated_sharpe_compute',      keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'deflated-sharpe',   descKey: 'shortcut.deflated_sharpe_compute',      actionKey: 'tv:deflated-sharpe-compute' },
    { id: 'deflated_sharpe_sweep',        keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'deflated-sharpe',   descKey: 'shortcut.deflated_sharpe_sweep',        actionKey: 'tv:deflated-sharpe-sweep' },
    { id: 'cup_and_handle_detect',        keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cup-and-handle',    descKey: 'shortcut.cup_and_handle_detect',        actionKey: 'tv:cup-and-handle-detect' },
    { id: 'cup_and_handle_demo',          keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cup-and-handle',    descKey: 'shortcut.cup_and_handle_demo',          actionKey: 'tv:cup-and-handle-demo' },
    { id: 'iv_rank_compute',              keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'iv-rank',           descKey: 'shortcut.iv_rank_compute',              actionKey: 'tv:iv-rank-compute' },
    { id: 'iv_rank_demo',                 keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'iv-rank',           descKey: 'shortcut.iv_rank_demo',                 actionKey: 'tv:iv-rank-demo' },
    { id: 'market_impact_analyze',        keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'market-impact',     descKey: 'shortcut.market_impact_analyze',        actionKey: 'tv:market-impact-analyze' },
    { id: 'market_impact_demo',           keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'market-impact',     descKey: 'shortcut.market_impact_demo',           actionKey: 'tv:market-impact-demo' },
    { id: 'liquidity_analyze',            keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'liquidity',         descKey: 'shortcut.liquidity_analyze',            actionKey: 'tv:liquidity-analyze' },
    { id: 'liquidity_demo',               keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'liquidity',         descKey: 'shortcut.liquidity_demo',               actionKey: 'tv:liquidity-demo' },
    { id: 'intraday_heatmap_build',       keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'intraday-heatmap',  descKey: 'shortcut.intraday_heatmap_build',       actionKey: 'tv:intraday-heatmap-build' },
    { id: 'intraday_heatmap_demo',        keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'intraday-heatmap',  descKey: 'shortcut.intraday_heatmap_demo',        actionKey: 'tv:intraday-heatmap-demo' },
    { id: 'iv_backtest_run',              keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'iv-backtest',       descKey: 'shortcut.iv_backtest_run',              actionKey: 'tv:iv-backtest-run' },
    { id: 'iv_backtest_demo',             keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'iv-backtest',       descKey: 'shortcut.iv_backtest_demo',             actionKey: 'tv:iv-backtest-demo' },
    { id: 'obi_compute',                  keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'order-book-imbalance', descKey: 'shortcut.obi_compute',               actionKey: 'tv:obi-compute' },
    { id: 'cusum_detect',                 keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cusum',             descKey: 'shortcut.cusum_detect',                 actionKey: 'tv:cusum-detect' },
    { id: 'cusum_autofit',                keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cusum',             descKey: 'shortcut.cusum_autofit',                actionKey: 'tv:cusum-autofit' },
    { id: 'order_flow_classify',          keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'order-flow',        descKey: 'shortcut.order_flow_classify',          actionKey: 'tv:order-flow-classify' },
    { id: 'order_flow_demo',              keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'order-flow',        descKey: 'shortcut.order_flow_demo',              actionKey: 'tv:order-flow-demo' },
    { id: 'vwap_slippage_analyze',        keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vwap-slippage',     descKey: 'shortcut.vwap_slippage_analyze',        actionKey: 'tv:vwap-slippage-analyze' },
    { id: 'vwap_slippage_demo',           keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vwap-slippage',     descKey: 'shortcut.vwap_slippage_demo',           actionKey: 'tv:vwap-slippage-demo' },
    { id: 'per_symbol_slippage_run',      keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'per-symbol-slippage', descKey: 'shortcut.per_symbol_slippage_run',    actionKey: 'tv:per-symbol-slippage-run' },
    { id: 'per_symbol_slippage_demo',     keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'per-symbol-slippage', descKey: 'shortcut.per_symbol_slippage_demo',   actionKey: 'tv:per-symbol-slippage-demo' },
    { id: 'order_staleness_evaluate',     keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'order-staleness',   descKey: 'shortcut.order_staleness_evaluate',     actionKey: 'tv:order-staleness-evaluate' },
    { id: 'order_staleness_demo',         keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'order-staleness',   descKey: 'shortcut.order_staleness_demo',         actionKey: 'tv:order-staleness-demo' },
    { id: 'mood_refresh',                 keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'mood',              descKey: 'shortcut.mood_refresh',                 actionKey: 'tv:mood-refresh' },
    { id: 'forecast_run',                 keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'forecast',          descKey: 'shortcut.forecast_run',                 actionKey: 'tv:forecast-run' },
    { id: 'cohort_tilt_run',              keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cohort-tilt',       descKey: 'shortcut.cohort_tilt_run',              actionKey: 'tv:cohort-tilt-run' },
    { id: 'setups_by_setup_run',          keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'setups-by-setup',   descKey: 'shortcut.setups_by_setup_run',          actionKey: 'tv:setups-by-setup-run' },
    { id: 'second_order_greeks_run',      keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'second-order-greeks', descKey: 'shortcut.second_order_greeks_run',    actionKey: 'tv:second-order-greeks-run' },
    { id: 'forward_vol_run',              keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'forward-vol',       descKey: 'shortcut.forward_vol_run',              actionKey: 'tv:forward-vol-run' },
    { id: 'yield_curve_pca_run',          keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'yield-curve-pca',   descKey: 'shortcut.yield_curve_pca_run',          actionKey: 'tv:yield-curve-pca-run' },
    { id: 'dividend_calendar_run',        keys: { key: 'f', meta: false, ctrl: false, shift: false, alt: false }, scope: 'dividend-calendar', descKey: 'shortcut.dividend_calendar_run',        actionKey: 'tv:dividend-calendar-run' },
    { id: 'signal_decomposition_run',     keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'signal-decomposition', descKey: 'shortcut.signal_decomposition_run', actionKey: 'tv:signal-decomposition-run' },
    { id: 'rr_butterfly_run',             keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'rr-butterfly',      descKey: 'shortcut.rr_butterfly_run',             actionKey: 'tv:rr-butterfly-run' },
    { id: 'cov_denoiser_run',             keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cov-denoiser',      descKey: 'shortcut.cov_denoiser_run',             actionKey: 'tv:cov-denoiser-run' },
    { id: 'almgren_chriss_run',           keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'almgren-chriss',    descKey: 'shortcut.almgren_chriss_run',           actionKey: 'tv:almgren-chriss-run' },
    { id: 'almgren_chriss_frontier',      keys: { key: 'f', meta: false, ctrl: false, shift: false, alt: false }, scope: 'almgren-chriss',    descKey: 'shortcut.almgren_chriss_frontier',      actionKey: 'tv:almgren-chriss-frontier' },
    { id: 'implementation_shortfall_run', keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'implementation-shortfall', descKey: 'shortcut.implementation_shortfall_run', actionKey: 'tv:implementation-shortfall-run' },
    { id: 'spread_tracker_run',           keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'spread-tracker',    descKey: 'shortcut.spread_tracker_run',           actionKey: 'tv:spread-tracker-run' },
    { id: 'spread_tracker_demo',          keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'spread-tracker',    descKey: 'shortcut.spread_tracker_demo',          actionKey: 'tv:spread-tracker-demo' },
    { id: 'open_type_run',                keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'open-type',         descKey: 'shortcut.open_type_run',                actionKey: 'tv:open-type-run' },
    { id: 'market_profile_run',           keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'market-profile',    descKey: 'shortcut.market_profile_run',           actionKey: 'tv:market-profile-run' },
    { id: 'market_profile_demo',          keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'market-profile',    descKey: 'shortcut.market_profile_demo',          actionKey: 'tv:market-profile-demo' },
    { id: 'pyramid_run',                  keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'pyramid',           descKey: 'shortcut.pyramid_run',                  actionKey: 'tv:pyramid-run' },
    { id: 'ha_reversal_run',              keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'ha-reversal',       descKey: 'shortcut.ha_reversal_run',              actionKey: 'tv:ha-reversal-run' },
    { id: 'ha_reversal_demo',             keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'ha-reversal',       descKey: 'shortcut.ha_reversal_demo',             actionKey: 'tv:ha-reversal-demo' },
    { id: 'three_bar_reversal_run',       keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'three-bar-reversal', descKey: 'shortcut.three_bar_reversal_run',      actionKey: 'tv:three-bar-reversal-run' },
    { id: 'three_bar_reversal_demo',      keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'three-bar-reversal', descKey: 'shortcut.three_bar_reversal_demo',     actionKey: 'tv:three-bar-reversal-demo' },
    { id: 'range_expansion_run',          keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'range-expansion',   descKey: 'shortcut.range_expansion_run',          actionKey: 'tv:range-expansion-run' },
    { id: 'range_expansion_demo',         keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'range-expansion',   descKey: 'shortcut.range_expansion_demo',         actionKey: 'tv:range-expansion-demo' },
    { id: 'alligator_run',                keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'alligator',         descKey: 'shortcut.alligator_run',                actionKey: 'tv:alligator-run' },
    { id: 'alligator_demo',               keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'alligator',         descKey: 'shortcut.alligator_demo',               actionKey: 'tv:alligator-demo' },
    { id: 'demarker_run',                 keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'demarker',          descKey: 'shortcut.demarker_run',                 actionKey: 'tv:demarker-run' },
    { id: 'demarker_demo',                keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'demarker',          descKey: 'shortcut.demarker_demo',                actionKey: 'tv:demarker-demo' },
    { id: 'murrey_math_run',              keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'murrey-math',       descKey: 'shortcut.murrey_math_run',              actionKey: 'tv:murrey-math-run' },
    { id: 'murrey_math_demo',             keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'murrey-math',       descKey: 'shortcut.murrey_math_demo',             actionKey: 'tv:murrey-math-demo' },
    { id: 'demark_pivots_run',            keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'demark-pivots',     descKey: 'shortcut.demark_pivots_run',            actionKey: 'tv:demark-pivots-run' },
    { id: 'cypher_pattern_run',           keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cypher-pattern',    descKey: 'shortcut.cypher_pattern_run',           actionKey: 'tv:cypher-pattern-run' },
    { id: 'cypher_pattern_demo',          keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cypher-pattern',    descKey: 'shortcut.cypher_pattern_demo',          actionKey: 'tv:cypher-pattern-demo' },
    { id: 'footprint_run',                keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'footprint',         descKey: 'shortcut.footprint_run',                actionKey: 'tv:footprint-run' },
    { id: 'footprint_demo',               keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'footprint',         descKey: 'shortcut.footprint_demo',               actionKey: 'tv:footprint-demo' },
    { id: 'stress_test_run',              keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'stress-test',       descKey: 'shortcut.stress_test_run',              actionKey: 'tv:stress-test-run' },
    { id: 'stress_test_demo',             keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'stress-test',       descKey: 'shortcut.stress_test_demo',             actionKey: 'tv:stress-test-demo' },
    { id: 'chandelier_stop_run',          keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'chandelier-stop',   descKey: 'shortcut.chandelier_stop_run',          actionKey: 'tv:chandelier-stop-run' },
    { id: 'chandelier_stop_demo',         keys: { key: 'l', meta: false, ctrl: false, shift: false, alt: false }, scope: 'chandelier-stop',   descKey: 'shortcut.chandelier_stop_demo',         actionKey: 'tv:chandelier-stop-demo' },
    { id: 'triple_screen_run',            keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'triple-screen',     descKey: 'shortcut.triple_screen_run',            actionKey: 'tv:triple-screen-run' },
    { id: 'daily_loss_limit_run',         keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'daily-loss-limit',  descKey: 'shortcut.daily_loss_limit_run',         actionKey: 'tv:daily-loss-limit-run' },
    { id: 'drawdown_throttle_run',        keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'drawdown-throttle', descKey: 'shortcut.drawdown_throttle_run',        actionKey: 'tv:drawdown-throttle-run' },
    { id: 'goal_tracker_run',             keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'goal-tracker',      descKey: 'shortcut.goal_tracker_run',             actionKey: 'tv:goal-tracker-run' },
    { id: 'trade_plan_checklist_run',     keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'trade-plan-checklist', descKey: 'shortcut.trade_plan_checklist_run', actionKey: 'tv:trade-plan-checklist-run' },
    { id: 'regime_equity_run',            keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'regime-equity',     descKey: 'shortcut.regime_equity_run',            actionKey: 'tv:regime-equity-run' },
    { id: 'vol_stop_close_run',           keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vol-stop-close',    descKey: 'shortcut.vol_stop_close_run',           actionKey: 'tv:vol-stop-close-run' },
    { id: 'time_in_force_run',            keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'time-in-force',     descKey: 'shortcut.time_in_force_run',            actionKey: 'tv:time-in-force-run' },
    { id: 'time_in_force_snap_now',       keys: { key: 'n', meta: false, ctrl: false, shift: false, alt: false }, scope: 'time-in-force',     descKey: 'shortcut.time_in_force_snap_now',       actionKey: 'tv:time-in-force-snap-now' },
    { id: 'clusters_trade_features_run',  keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'clusters-trade-features', descKey: 'shortcut.clusters_trade_features_run', actionKey: 'tv:clusters-trade-features-run' },
    { id: 'clusters_correlation_run',     keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'clusters-correlation',    descKey: 'shortcut.clusters_correlation_run',    actionKey: 'tv:clusters-correlation-run' },
    { id: 'choppiness_run',               keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'choppiness',        descKey: 'shortcut.choppiness_run',               actionKey: 'tv:choppiness-run' },
    { id: 'var_estimator_run',            keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'var-estimator',     descKey: 'shortcut.var_estimator_run',            actionKey: 'tv:var-estimator-run' },
    { id: 'mc_trades_run',                keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'mc-trades',         descKey: 'shortcut.mc_trades_run',                actionKey: 'tv:mc-trades-run' },
    { id: 'commission_optimizer_run',     keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'commission-optimizer', descKey: 'shortcut.commission_optimizer_run',  actionKey: 'tv:commission-optimizer-run' },
    { id: 'margin_runway_run',            keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'margin-runway',     descKey: 'shortcut.margin_runway_run',            actionKey: 'tv:margin-runway-run' },
    { id: 'risk_parity_run',              keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'risk-parity',       descKey: 'shortcut.risk_parity_run',              actionKey: 'tv:risk-parity-run' },
    { id: 'risk_on_off_run',              keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'risk-on-off',       descKey: 'shortcut.risk_on_off_run',              actionKey: 'tv:risk-on-off-run' },
    { id: 'risk_reward_run',              keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'risk-reward',       descKey: 'shortcut.risk_reward_run',              actionKey: 'tv:risk-reward-run' },
    { id: 'tax_loss_harvest_run',         keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'tax-loss-harvest',  descKey: 'shortcut.tax_loss_harvest_run',         actionKey: 'tv:tax-loss-harvest-run' },
    { id: 'wash_sale_run',                keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'wash-sale',         descKey: 'shortcut.wash_sale_run',                actionKey: 'tv:wash-sale-run' },
    { id: 'buying_power_run',             keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'buying-power',      descKey: 'shortcut.buying_power_run',             actionKey: 'tv:buying-power-run' },
    { id: 'margin_call_run',              keys: { key: 'e', meta: false, ctrl: false, shift: false, alt: false }, scope: 'margin-call',       descKey: 'shortcut.margin_call_run',              actionKey: 'tv:margin-call-run' },
    { id: 'vix_term_structure_run',       keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vix-term-structure', descKey: 'shortcut.vix_term_structure_run',      actionKey: 'tv:vix-term-structure-run' },
    { id: 'currency_exposure_run',        keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'currency-exposure', descKey: 'shortcut.currency_exposure_run',        actionKey: 'tv:currency-exposure-run' },
    { id: 'bond_duration_run',            keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'bond-duration',     descKey: 'shortcut.bond_duration_run',            actionKey: 'tv:bond-duration-run' },
    { id: 'bond_duration_build',          keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'bond-duration',     descKey: 'shortcut.bond_duration_build',          actionKey: 'tv:bond-duration-build' },
    { id: 'carry_score_run',              keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'carry-score',       descKey: 'shortcut.carry_score_run',              actionKey: 'tv:carry-score-run' },
    { id: 'yield_curve_run',              keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'yield-curve',       descKey: 'shortcut.yield_curve_run',              actionKey: 'tv:yield-curve-run' },
    { id: 'cost_basis_run',               keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cost-basis',        descKey: 'shortcut.cost_basis_run',               actionKey: 'tv:cost-basis-run' },
    { id: 'cost_basis_opt',               keys: { key: 'o', meta: false, ctrl: false, shift: false, alt: false }, scope: 'cost-basis',        descKey: 'shortcut.cost_basis_opt',               actionKey: 'tv:cost-basis-opt' },
    { id: 'stop_loss_backtest_run',       keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'stop-loss-backtest', descKey: 'shortcut.stop_loss_backtest_run',      actionKey: 'tv:stop-loss-backtest-run' },
    { id: 'futures_roll_run',             keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'futures-roll',      descKey: 'shortcut.futures_roll_run',             actionKey: 'tv:futures-roll-run' },
    { id: 'heatmap_dow_hour_run',         keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'heatmap-dow-hour',  descKey: 'shortcut.heatmap_dow_hour_run',         actionKey: 'tv:heatmap-dow-hour-run' },
    { id: 'atr_cone_run',                 keys: { key: 'p', meta: false, ctrl: false, shift: false, alt: false }, scope: 'atr-cone',          descKey: 'shortcut.atr_cone_run',                 actionKey: 'tv:atr-cone-run' },
    { id: 'round_levels_run',             keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'round-levels',      descKey: 'shortcut.round_levels_run',             actionKey: 'tv:round-levels-run' },
    { id: 'kyles_lambda_run',             keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'kyles-lambda',      descKey: 'shortcut.kyles_lambda_run',             actionKey: 'tv:kyles-lambda-run' },
    { id: 'hawkes_run',                   keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'hawkes',            descKey: 'shortcut.hawkes_run',                   actionKey: 'tv:hawkes-run' },
    { id: 'kagi_run',                     keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'kagi',              descKey: 'shortcut.kagi_run',                     actionKey: 'tv:kagi-run' },
    { id: 'risk_parity_solver_run',       keys: { key: 's', meta: false, ctrl: false, shift: false, alt: false }, scope: 'risk-parity-solver', descKey: 'shortcut.risk_parity_solver_run',      actionKey: 'tv:risk-parity-solver-run' },
    { id: 'volume_at_price_run',          keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vap',               descKey: 'shortcut.volume_at_price_run',          actionKey: 'tv:volume-at-price-run' },
    { id: 'herfindahl_run',               keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'hhi',               descKey: 'shortcut.herfindahl_run',               actionKey: 'tv:herfindahl-run' },
    { id: 'roll_spread_run',              keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'roll-spread',       descKey: 'shortcut.roll_spread_run',              actionKey: 'tv:roll-spread-run' },
    { id: 'three_line_break_run',         keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'tlb',               descKey: 'shortcut.three_line_break_run',         actionKey: 'tv:three-line-break-run' },
    { id: 'momentum_crash_run',           keys: { key: 'm', meta: false, ctrl: false, shift: false, alt: false }, scope: 'mcp',               descKey: 'shortcut.momentum_crash_run',           actionKey: 'tv:momentum-crash-run' },
    { id: 'effective_spread_run',         keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'eff-spread',        descKey: 'shortcut.effective_spread_run',         actionKey: 'tv:effective-spread-run' },
    { id: 'weighted_midprice_run',        keys: { key: 'c', meta: false, ctrl: false, shift: false, alt: false }, scope: 'wmp',               descKey: 'shortcut.weighted_midprice_run',        actionKey: 'tv:weighted-midprice-run' },
    { id: 'marginal_var_run',             keys: { key: 'a', meta: false, ctrl: false, shift: false, alt: false }, scope: 'mvar',              descKey: 'shortcut.marginal_var_run',             actionKey: 'tv:marginal-var-run' },
    { id: 'range_bar_run',                keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'range-bar',         descKey: 'shortcut.range_bar_run',                actionKey: 'tv:range-bar-run' },
    { id: 'tick_bar_run',                 keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'tick-bar',          descKey: 'shortcut.tick_bar_run',                 actionKey: 'tv:tick-bar-run' },
    { id: 'volume_bar_run',               keys: { key: 'b', meta: false, ctrl: false, shift: false, alt: false }, scope: 'vol-bar',           descKey: 'shortcut.volume_bar_run',               actionKey: 'tv:volume-bar-run' },
    { id: 'abc_pattern_run',              keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'abc-pattern',       descKey: 'shortcut.abc_pattern_run',              actionKey: 'tv:abc-pattern-run' },
    { id: 'absorption_run',               keys: { key: 'd', meta: false, ctrl: false, shift: false, alt: false }, scope: 'absorption',        descKey: 'shortcut.absorption_run',               actionKey: 'tv:absorption-run' },
    { id: 'live_refresh',         keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'live',       descKey: 'shortcut.live_refresh',         actionKey: 'tv:live-refresh' },
    { id: 'trades_refresh',       keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'trades',     descKey: 'shortcut.trades_refresh',       actionKey: 'tv:trades-refresh' },
    { id: 'journal_refresh',      keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'journal',    descKey: 'shortcut.journal_refresh',      actionKey: 'tv:journal-refresh' },
    { id: 'watchlists_refresh',   keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'watchlists', descKey: 'shortcut.watchlists_refresh',   actionKey: 'tv:watchlists-refresh' },
    { id: 'webull_refresh',       keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'webull',     descKey: 'shortcut.webull_refresh',       actionKey: 'tv:webull-refresh' },
    { id: 'charts_refresh',       keys: { key: 'r', meta: false, ctrl: false, shift: false, alt: false }, scope: 'charts',     descKey: 'shortcut.charts_refresh',       actionKey: 'tv:charts-refresh' },
    // Quick-nav globals (Cmd/Ctrl + Option/Alt + letter): zero-conflict
    // with browser defaults; jumps directly to high-traffic views.
    { id: 'nav_trades',      keys: { key: 't', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_trades',      actionKey: 'tv:nav-trades' },
    { id: 'nav_journal',     keys: { key: 'j', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_journal',     actionKey: 'tv:nav-journal' },
    { id: 'nav_dashboard',   keys: { key: 'd', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_dashboard',   actionKey: 'tv:nav-dashboard' },
    { id: 'nav_watchlists',  keys: { key: 'w', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_watchlists',  actionKey: 'tv:nav-watchlists' },
    { id: 'nav_charts',      keys: { key: 'c', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_charts',      actionKey: 'tv:nav-charts' },
    { id: 'nav_live',        keys: { key: 'l', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_live',        actionKey: 'tv:nav-live' },
    { id: 'nav_reports',     keys: { key: 'r', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_reports',     actionKey: 'tv:nav-reports' },
    { id: 'nav_scanner',     keys: { key: 'm', meta: true, ctrl: true, shift: false, alt: true }, scope: 'global', descKey: 'shortcut.nav_scanner',     actionKey: 'tv:nav-scanner' },
];

// Whether a DOM-style keydown event satisfies a shortcut keys spec.
// `meta` OR `ctrl` matches both Mac and PC users — set both `true` in
// the keys spec to "fire if either Cmd or Ctrl is down". Set `meta: true,
// ctrl: false` to be strict Mac-only.
export function matches(e, sc) {
    if (!e || !sc || !sc.keys) return false;
    const k = sc.keys;
    if (k.key == null) return false;
    if (typeof e.key !== 'string') return false;
    if (e.key.toLowerCase() !== String(k.key).toLowerCase()) return false;
    // Modifier match policy: meta/ctrl are OR'd if both required keys are
    // true (cross-platform); otherwise strict equality.
    if (k.meta && k.ctrl) {
        if (!(e.metaKey || e.ctrlKey)) return false;
    } else {
        if ((k.meta  || false) !== !!e.metaKey) return false;
        if ((k.ctrl  || false) !== !!e.ctrlKey) return false;
    }
    if ((k.shift || false) !== !!e.shiftKey) return false;
    if ((k.alt   || false) !== !!e.altKey)   return false;
    return true;
}

// Human-readable key chip. macOS-style glyphs when available.
export function formatKey(sc, isMac = true) {
    if (!sc || !sc.keys) return '';
    const k = sc.keys;
    const parts = [];
    if (k.meta && k.ctrl) parts.push(isMac ? '⌘' : 'Ctrl');
    else {
        if (k.ctrl)  parts.push(isMac ? '⌃' : 'Ctrl');
        if (k.meta)  parts.push(isMac ? '⌘' : 'Win');
    }
    if (k.shift) parts.push(isMac ? '⇧' : 'Shift');
    if (k.alt)   parts.push(isMac ? '⌥' : 'Alt');
    parts.push(prettyKey(k.key));
    return parts.join(isMac ? '' : '+');
}

function prettyKey(k) {
    if (!k) return '';
    if (k === ' ' || k.toLowerCase() === 'space') return '␣';
    if (k.length === 1) return k.toUpperCase();
    return k;
}

// Find first registered shortcut whose keys match the event, scoped by
// `currentScope` ('global' shortcuts always match; others only match
// when current scope === sc.scope).
export function findMatch(event, shortcuts, currentScope = 'global') {
    if (!Array.isArray(shortcuts)) return null;
    for (const sc of shortcuts) {
        if (!sc.enabled && sc.enabled !== undefined) continue;
        if (sc.scope !== 'global' && sc.scope !== currentScope) continue;
        if (matches(event, sc)) return sc;
    }
    return null;
}

// localStorage-backed: load user overrides on top of defaults. Each
// override is keyed by shortcut id and replaces `keys` (not the whole
// entry — descKey + actionKey stay from defaults so user can't break
// the registry by deleting them).
export function loadShortcuts(getItem) {
    const get = getItem || ((typeof localStorage !== 'undefined') ? (k => localStorage.getItem(k)) : () => null);
    let saved = {};
    try {
        const raw = get(LS_KEY);
        if (raw) {
            const obj = JSON.parse(raw);
            if (obj && obj.version === VERSION && obj.overrides && typeof obj.overrides === 'object') {
                saved = obj.overrides;
            }
        }
    } catch { /* malformed → ignore */ }
    return DEFAULT_SHORTCUTS.map(sc =>
        saved[sc.id] ? { ...sc, keys: saved[sc.id] } : { ...sc });
}

export function saveOverrides(overrides, setItem) {
    const set = setItem || ((typeof localStorage !== 'undefined')
        ? ((k, v) => localStorage.setItem(k, v))
        : () => {});
    try { set(LS_KEY, JSON.stringify({ version: VERSION, overrides })); }
    catch { /* private mode */ }
}

// Should the shortcut fire even when the user is typing? Most shortcuts
// should NOT — leave room for the user to type 'k' in a textbox. But a
// few (Escape, Cmd+K, Cmd+/) DO want to fire from inside text fields
// because they are how you EXIT the field.
export function firesInEditableContext(sc) {
    if (!sc) return false;
    if (sc.id === 'escape') return true;
    // Cmd+K-like (meta && ctrl-tolerant) always fires.
    return !!(sc.keys && sc.keys.meta && sc.keys.ctrl);
}

// True if the event target is a text-entry element (input/textarea/select/
// contentEditable). Used together with `firesInEditableContext` to gate
// non-modifier shortcuts.
export function isTextEntryTarget(t) {
    if (!t) return false;
    const tag = (t.tagName || '').toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select') return true;
    if (t.isContentEditable) return true;
    return false;
}
