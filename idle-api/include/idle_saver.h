/* SPDX-License-Identifier: Apache-2.0
 * Copyright 2026 IdleScreen
 *
 * idle_saver.h — C ABI for IdleScreen screensaver plugins.
 *
 * Any language that can build a shared library (.so) with C linkage can
 * write a screensaver. Rust plugins may instead use the legacy
 * `create_screensaver`/`destroy_screensaver` entry points, which return a
 * boxed trait object; foreign languages must use the ops-table surface below.
 *
 * ── Required symbols ────────────────────────────────────────────────────
 *
 *   uint32_t idle_api_version(void);
 *       Return IDLE_API_VERSION. The host refuses the plugin on mismatch.
 *
 *   const IdleSaverOps *idle_saver_ops(void);
 *       Return a pointer to a static (or otherwise permanently-resident)
 *       ops table. The host dereferences it only while the library is
 *       loaded. When this symbol exists it is used INSTEAD of the legacy
 *       Rust create_screensaver/destroy_screensaver pair.
 *
 * ── Calling contract ────────────────────────────────────────────────────
 *
 *  - ops->create() allocates plugin state and returns an opaque ctx.
 *    The host passes ctx to every callback and never dereferences it.
 *  - ops->destroy(ctx) frees that state exactly once, on unload.
 *  - ops->draw(ctx, cells, cols, rows) receives a host-owned buffer of
 *    cols*rows IdleCells to paint. The pointer is valid only for the call;
 *    do not retain it. Unset cells render as space with default colors.
 *  - ops->spotlights(ctx, out, capacity) writes up to `capacity` entries and
 *    returns the number written (a count > capacity is clamped by the host).
 *  - Optional members may be NULL; the host uses the trait default.
 *  - All callbacks run on the host's render thread — single-threaded.
 *  - Plugin code must not unwind across the boundary; report failure by
 *    returning NULL from create().
 */

#ifndef IDLE_SAVER_H
#define IDLE_SAVER_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Matches idle_api::API_VERSION — bump on breaking ABI change. */
#define IDLE_API_VERSION 1u

/* C-ABI mirror of idle_api::TerminalCell (repr(C)). */
typedef struct {
    uint32_t ch;    /* Unicode scalar value (char32_t); invalid → space */
    uint8_t  fg[3]; /* foreground RGB */
    uint8_t  bg[3]; /* background RGB */
    uint8_t  bold;  /* 0 = normal, nonzero = bold (double-width glyph) */
} IdleCell;

/* C-ABI mirror of idle_api::GpuSpotlight (repr(C), 7×f32 + pad). */
typedef struct {
    float origin_x_ratio;
    float color_r;
    float color_g;
    float color_b;
    float angle;
    float spread;
    float speed;
    float _pad;
} IdleGpuSpotlight;

typedef struct {
    /* Value of IDLE_API_VERSION the plugin was built against. */
    uint32_t abi_version;

    /* Allocate plugin state; return opaque ctx. NULL = failure. */
    void *(*create)(void);
    /* Free plugin state; called exactly once on unload. */
    void (*destroy)(void *ctx);

    /* Optional: called after creation and on resize. */
    void (*init)(void *ctx, uint32_t cols, uint32_t rows);
    /* Advance the simulation by dt_seconds. */
    void (*update)(void *ctx, double dt_seconds, uint32_t cols, uint32_t rows);
    /* Optional: sub-frame timing hook. */
    void (*update_frame_time)(void *ctx, double dt_seconds);
    /* Paint the cols*rows cell buffer. */
    void (*draw)(void *ctx, IdleCell *cells, uint32_t cols, uint32_t rows);
    /* Optional: nonzero enables scanline postfx. */
    uint8_t (*has_scanlines)(void *ctx);
    /* Optional: write up to capacity spotlights; return count written. */
    uint32_t (*spotlights)(void *ctx, IdleGpuSpotlight *out, uint32_t capacity);
} IdleSaverOps;

uint32_t idle_api_version(void);
const IdleSaverOps *idle_saver_ops(void);

#ifdef __cplusplus
}
#endif

#endif /* IDLE_SAVER_H */
