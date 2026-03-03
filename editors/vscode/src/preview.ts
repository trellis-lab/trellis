import * as vscode from 'vscode';
import * as path from 'path';
import { buildConfigJson, HostMessage, WebviewMessage } from './wasm-bridge';

/**
 * Manages the singleton Trellis preview webview panel.
 *
 * Only one preview panel is open at a time.  Calling `createOrShow` while a
 * panel already exists brings that panel to the foreground.
 */
export class PreviewPanel {
    public static readonly VIEW_TYPE = 'trellisPreview';

    private static _current: PreviewPanel | undefined;

    private readonly _panel: vscode.WebviewPanel;
    private readonly _extensionUri: vscode.Uri;
    private _disposables: vscode.Disposable[] = [];
    private _wasmReady = false;

    // ─── Public API ───────────────────────────────────────────────────────────

    /** Create the panel (or reveal it if it already exists). */
    public static createOrShow(
        extensionUri: vscode.Uri,
        initialSource: string,
        column: vscode.ViewColumn = vscode.ViewColumn.Beside,
    ): void {
        if (PreviewPanel._current) {
            PreviewPanel._current._panel.reveal(column);
            PreviewPanel._current._sendUpdate(initialSource);
            return;
        }
        const panel = vscode.window.createWebviewPanel(
            PreviewPanel.VIEW_TYPE,
            'Trellis Preview',
            column,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [
                    vscode.Uri.joinPath(extensionUri, 'out'),
                    vscode.Uri.joinPath(extensionUri, 'media'),
                    vscode.Uri.joinPath(extensionUri, 'wasm'),
                ],
            },
        );
        PreviewPanel._current = new PreviewPanel(panel, extensionUri, initialSource);
    }

    /** Push a new Mermaid source string to the active preview, if any. */
    public static update(source: string): void {
        PreviewPanel._current?._sendUpdate(source);
    }

    /** Dispose the active preview panel. */
    public static dispose(): void {
        PreviewPanel._current?.dispose();
    }

    // ─── Construction / teardown ──────────────────────────────────────────────

    private constructor(
        panel: vscode.WebviewPanel,
        extensionUri: vscode.Uri,
        initialSource: string,
    ) {
        this._panel = panel;
        this._extensionUri = extensionUri;

        this._panel.webview.html = this._buildHtml(this._panel.webview);

        // Handle messages from the webview
        this._panel.webview.onDidReceiveMessage(
            (msg: WebviewMessage) => this._onWebviewMessage(msg),
            null,
            this._disposables,
        );

        // When the panel is closed, clean up
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        // Send the WASM URIs once the webview is ready, then immediately
        // push the initial source.  We wait for the 'ready' ACK from the
        // webview before sending the update.
        this._sendInit(initialSource);
    }

    private dispose(): void {
        PreviewPanel._current = undefined;
        this._panel.dispose();
        for (const d of this._disposables) {
            d.dispose();
        }
        this._disposables = [];
    }

    // ─── Message helpers ──────────────────────────────────────────────────────

    private _sendInit(initialSource: string): void {
        const webview = this._panel.webview;

        const wasmModuleUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._extensionUri, 'wasm', 'trellis_wasm.js'),
        ).toString();
        const wasmBinaryUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._extensionUri, 'wasm', 'trellis_wasm_bg.wasm'),
        ).toString();

        const initMsg: HostMessage = {
            type: 'init',
            wasmModuleUri,
            wasmBinaryUri,
        };
        this._panel.webview.postMessage(initMsg).then(() => {
            // Send first render right after init
            this._sendUpdate(initialSource);
        });
    }

    private _sendUpdate(source: string): void {
        if (!this._panel.visible) {
            return;
        }
        const msg: HostMessage = {
            type: 'update',
            source,
            configJson: buildConfigJson(),
        };
        this._panel.webview.postMessage(msg);
    }

    private _onWebviewMessage(msg: WebviewMessage): void {
        switch (msg.type) {
            case 'renderOk': {
                // Optionally show metrics in the status bar – for now just log
                let metrics: Record<string, unknown> = {};
                try {
                    const parsed = JSON.parse(msg.metricsJson) as { metrics?: Record<string, unknown> };
                    metrics = parsed.metrics ?? {};
                } catch {
                    // ignore
                }
                const nodes = metrics['nodes'] ?? '?';
                const edges = metrics['edges'] ?? '?';
                const ms = metrics['render_ms'] ?? '?';
                this._panel.title = `Trellis Preview (${nodes}n ${edges}e ${ms}ms)`;
                break;
            }
            case 'renderError':
                vscode.window.showErrorMessage(`Trellis render error: ${msg.message}`);
                break;
        }
    }

    // ─── Webview HTML ─────────────────────────────────────────────────────────

    private _buildHtml(webview: vscode.Webview): string {
        // Compute a nonce for the Content-Security-Policy
        const nonce = getNonce();

        // Load the preview HTML template from media/preview.html
        const previewHtmlUri = vscode.Uri.joinPath(this._extensionUri, 'media', 'preview.html');
        // We inline the template directly – avoids async FS reads in the constructor
        return getPreviewHtml(webview, this._extensionUri, nonce);
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function getNonce(): string {
    let text = '';
    const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    for (let i = 0; i < 32; i++) {
        text += possible.charAt(Math.floor(Math.random() * possible.length));
    }
    return text;
}

/**
 * Return the full HTML document for the preview webview.
 * The actual rendering logic lives in `media/preview.html` (inlined here via
 * template literal so we avoid async FS reads at panel-creation time).
 */
function getPreviewHtml(
    webview: vscode.Webview,
    extensionUri: vscode.Uri,
    nonce: string,
): string {
    // Build webview-safe URIs for local assets
    const styleUri = webview.asWebviewUri(
        vscode.Uri.joinPath(extensionUri, 'media', 'preview.css'),
    );

    // CSP: allow scripts from the extension + inline scripts with this nonce;
    //      allow wasm-unsafe-eval for the WASM binary.
    const csp = [
        `default-src 'none'`,
        `style-src ${webview.cspSource} 'unsafe-inline'`,
        `script-src 'nonce-${nonce}' 'wasm-unsafe-eval'`,
        `img-src ${webview.cspSource} data:`,
        `connect-src ${webview.cspSource}`,
    ].join('; ');

    return /* html */ `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta http-equiv="Content-Security-Policy" content="${csp}" />
  <title>Trellis Preview</title>
  <style nonce="${nonce}">
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--vscode-editor-background, #1e1e1e);
      color: var(--vscode-editor-foreground, #d4d4d4);
      font-family: var(--vscode-font-family, sans-serif);
      height: 100vh;
      display: flex;
      flex-direction: column;
    }
    #toolbar {
      padding: 4px 8px;
      background: var(--vscode-sideBar-background, #252526);
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 12px;
      border-bottom: 1px solid var(--vscode-sideBarSectionHeader-border, #444);
    }
    #status { flex: 1; opacity: 0.7; }
    #diagram-container {
      flex: 1;
      overflow: auto;
      display: flex;
      align-items: flex-start;
      justify-content: center;
      padding: 16px;
    }
    #diagram-container svg {
      max-width: 100%;
      height: auto;
      background: white;
      border-radius: 4px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.4);
    }
    #error-box {
      display: none;
      margin: 16px;
      padding: 12px 16px;
      background: var(--vscode-inputValidation-errorBackground, #5a1d1d);
      border: 1px solid var(--vscode-inputValidation-errorBorder, #be1100);
      border-radius: 4px;
      font-family: var(--vscode-editor-font-family, monospace);
      font-size: 13px;
      white-space: pre-wrap;
    }
    .spinner {
      width: 32px; height: 32px;
      border: 3px solid var(--vscode-progressBar-background, #0078d4);
      border-top-color: transparent;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
      margin: auto;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
  </style>
</head>
<body>
  <div id="toolbar">
    <span id="status">Initialising Trellis renderer…</span>
  </div>
  <div id="error-box"></div>
  <div id="diagram-container">
    <div class="spinner" id="spinner"></div>
  </div>

  <script type="module" nonce="${nonce}">
    // ── VS Code API ──────────────────────────────────────────────────────────
    const vscode = acquireVsCodeApi();

    // ── State ────────────────────────────────────────────────────────────────
    let wasmModule = null;
    let pendingSource = null;
    let pendingConfig = null;

    const statusEl = document.getElementById('status');
    const spinnerEl = document.getElementById('spinner');
    const containerEl = document.getElementById('diagram-container');
    const errorEl = document.getElementById('error-box');

    function setStatus(text) { statusEl.textContent = text; }
    function showSpinner() { spinnerEl.style.display = 'block'; }
    function hideSpinner() { spinnerEl.style.display = 'none'; }
    function showError(msg) {
        errorEl.textContent = msg;
        errorEl.style.display = 'block';
    }
    function clearError() { errorEl.style.display = 'none'; errorEl.textContent = ''; }

    // ── Message handler ──────────────────────────────────────────────────────
    window.addEventListener('message', async (event) => {
        const msg = event.data;
        switch (msg.type) {
            case 'init':
                await initWasm(msg.wasmModuleUri, msg.wasmBinaryUri);
                break;
            case 'update':
                pendingSource = msg.source;
                pendingConfig = msg.configJson || '';
                if (wasmModule) {
                    await renderDiagram(pendingSource, pendingConfig);
                }
                break;
        }
    });

    // ── WASM initialisation ──────────────────────────────────────────────────
    async function initWasm(moduleUri, binaryUri) {
        setStatus('Loading WASM renderer…');
        showSpinner();
        try {
            // Dynamic import of the wasm-pack generated ES module
            const mod = await import(/* @vite-ignore */ moduleUri);
            // Initialise the WASM binary
            await mod.default(binaryUri);
            wasmModule = mod;
            hideSpinner();
            setStatus('Trellis ready');
            // Render any source that arrived before WASM was ready
            if (pendingSource !== null) {
                await renderDiagram(pendingSource, pendingConfig || '');
            }
        } catch (err) {
            hideSpinner();
            const message = String(err);
            setStatus('Failed to load WASM: ' + message);
            showError('WASM load error:\\n' + message);
            vscode.postMessage({ type: 'renderError', message });
        }
    }

    // ── Rendering ────────────────────────────────────────────────────────────
    async function renderDiagram(source, configJson) {
        if (!wasmModule) return;
        clearError();
        showSpinner();
        setStatus('Rendering…');
        try {
            // render_with_metrics returns JSON: { svg, metrics }
            const resultJson = wasmModule.render_with_metrics(
                source,
                configJson || null,
            );
            const result = JSON.parse(resultJson);
            displaySvg(result.svg);
            const m = result.metrics;
            setStatus(
                \`\${m.nodes} nodes · \${m.edges} edges · \${m.render_ms}ms\`
            );
            hideSpinner();
            vscode.postMessage({
                type: 'renderOk',
                metricsJson: resultJson,
            });
        } catch (err) {
            hideSpinner();
            const message = String(err);
            setStatus('Render error');
            showError(message);
            vscode.postMessage({ type: 'renderError', message });
        }
    }

    function displaySvg(svgString) {
        // Remove any previous SVG
        const existing = containerEl.querySelector('svg');
        if (existing) existing.remove();
        // Create a temporary container to parse the SVG
        const tmp = document.createElement('div');
        tmp.innerHTML = svgString;
        const svgEl = tmp.querySelector('svg');
        if (svgEl) {
            containerEl.appendChild(svgEl);
        }
    }
  </script>
</body>
</html>`;
}

// Export for testing
export { getPreviewHtml, getNonce };
