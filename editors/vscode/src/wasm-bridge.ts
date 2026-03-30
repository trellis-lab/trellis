/**
 * wasm-bridge.ts
 *
 * This module defines the message protocol between the VS Code extension host
 * and the preview webview.  The actual WASM loading and rendering happens
 * inside the webview (see media/preview.html), which runs in a browser-like
 * sandbox.  The extension host communicates with the webview by posting
 * strongly-typed messages defined here.
 *
 * Architecture:
 *   Extension host  ──postMessage──>  Webview
 *                   <──postMessage──
 *
 * The webview loads the wasm-pack `--target web` output from the extension's
 * `wasm/` directory, calls `trellis_wasm.render()`, and displays the SVG.
 */

// ─── Messages sent FROM the extension host TO the webview ────────────────────

/** Tell the webview to render a new Mermaid source string. */
export interface UpdateMessage {
    type: 'update';
    /** Raw Mermaid source code. */
    source: string;
    /** JSON-serialised `TrellisConfig` (optional – omit to use defaults). */
    configJson?: string;
}

/** Tell the webview which base URI to use for loading the WASM module. */
export interface InitMessage {
    type: 'init';
    /** Webview-safe URI for `trellis_wasm.js` (ES module entry point). */
    wasmModuleUri: string;
    /** Webview-safe URI for `trellis_wasm_bg.wasm` (binary). */
    wasmBinaryUri: string;
}

export type HostMessage = InitMessage | UpdateMessage;

// ─── Messages sent FROM the webview TO the extension host ────────────────────

/** The webview reports a successful render. */
export interface RenderOkMessage {
    type: 'renderOk';
    /** Serialised render metrics from `render_with_metrics()`. */
    metricsJson: string;
}

/** The webview reports a render or WASM error. */
export interface RenderErrorMessage {
    type: 'renderError';
    message: string;
}

export type WebviewMessage = RenderOkMessage | RenderErrorMessage;

// ─── Config snapshot sent with each update ────────────────────────────────────

import * as vscode from 'vscode';

/**
 * Build a partial `TrellisConfig` JSON string from the current VS Code
 * workspace settings.  Only settings that differ from Rust defaults are
 * included so the JSON stays minimal.
 */
export function buildConfigJson(): string {
    const cfg = vscode.workspace.getConfiguration('trellis');
    const partial: Record<string, unknown> = {};

    const cellSize = cfg.get<number>('cellSize');
    if (cellSize !== undefined && cellSize !== 10) {
        partial['cell_size'] = cellSize;
    }

    const showEdgeLabels = cfg.get<boolean>('showEdgeLabels');
    if (showEdgeLabels !== undefined) {
        partial['show_edge_labels'] = showEdgeLabels;
    }

    return Object.keys(partial).length > 0 ? JSON.stringify(partial) : '';
}
