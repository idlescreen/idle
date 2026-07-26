//! Env-var fallback for primary monitor cell bounds (IPC shim until channel exists).

use std::sync::{OnceLock, RwLock};

use super::bounds::MonitorCellBounds;

static ENV_PRIMARY_BOUNDS: OnceLock<RwLock<Option<MonitorCellBounds>>> = OnceLock::new();

fn env_bounds_cache() -> &'static RwLock<Option<MonitorCellBounds>> {
    ENV_PRIMARY_BOUNDS.get_or_init(|| RwLock::new(None))
}

pub(crate) fn cached_primary_bounds_from_env() -> Option<MonitorCellBounds> {
    if let Ok(read_guard) = env_bounds_cache().read()
        && let Some(bounds) = *read_guard
    {
        return Some(bounds);
    }
    let mut cache = env_bounds_cache()
        .write()
        .unwrap_or_else(|e| e.into_inner());
    if cache.is_none() {
        *cache = read_primary_bounds_from_env();
    }
    *cache
}

fn read_primary_bounds_from_env() -> Option<MonitorCellBounds> {
    let start_col = crate::env_var_first(&["IDLE_PRIMARY_START_COL"])?
        .parse()
        .ok()?;
    let end_col = crate::env_var_first(&["IDLE_PRIMARY_END_COL"])?
        .parse()
        .ok()?;
    let start_row = crate::env_var_first(&["IDLE_PRIMARY_START_ROW"])?
        .parse()
        .ok()?;
    let end_row = crate::env_var_first(&["IDLE_PRIMARY_END_ROW"])?
        .parse()
        .ok()?;
    if end_col <= start_col || end_row <= start_row {
        return None;
    }
    const MAX_GRID: usize = 16_384;
    if end_col > MAX_GRID || end_row > MAX_GRID {
        return None;
    }
    Some(MonitorCellBounds {
        start_col,
        end_col,
        start_row,
        end_row,
        is_primary: true,
    })
}

pub(crate) fn store_primary_bounds(bounds: MonitorCellBounds) {
    // SAFETY (Phase 4 note): the `unsafe std::env::set_var` calls below are a known
    // hazard — `std::env::set_var` is not thread-safe and the surrounding `unsafe`
    // blocks provide no actual safety guarantee. A follow-up Phase 4 agent working
    // on the daemon crate will replace this IPC mechanism with a thread-safe channel,
    // at which point these `unsafe` blocks and the env-var fallback in
    // `read_primary_bounds_from_env` can be removed entirely. Do not remove them yet.
    crate::set_env("IDLE_PRIMARY_START_COL", bounds.start_col.to_string());
    crate::set_env("IDLE_PRIMARY_END_COL", bounds.end_col.to_string());
    crate::set_env("IDLE_PRIMARY_START_ROW", bounds.start_row.to_string());
    crate::set_env("IDLE_PRIMARY_END_ROW", bounds.end_row.to_string());
    *env_bounds_cache()
        .write()
        .unwrap_or_else(|e| e.into_inner()) = Some(bounds);
}

pub(crate) fn clear_stored_primary_bounds() {
    // See `store_primary_bounds` for the Phase 4 hazard note.
    // SAFETY: same process-global env protocol as store; test/host teardown.
    unsafe {
        for k in [
            "IDLE_PRIMARY_START_COL",
            "IDLE_PRIMARY_END_COL",
            "IDLE_PRIMARY_START_ROW",
            "IDLE_PRIMARY_END_ROW",
        ] {
            std::env::remove_var(k);
        }
    }
    *env_bounds_cache()
        .write()
        .unwrap_or_else(|e| e.into_inner()) = None;
}
