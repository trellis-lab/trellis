/**
 * export.ts – PNG / SVG export from the preview panel.
 *
 * NOTE: Full export support (SVG for Premium, PNG for all tiers) is gated
 * behind licence validation and will be implemented in milestone Mc
 * (commercialisation).  This file is a placeholder that defines the public
 * API surface so that `preview.ts` can import it without compile errors.
 */

import * as vscode from 'vscode';

/** Supported export formats. */
export type ExportFormat = 'svg' | 'png';

/**
 * Prompt the user for a save path and write the diagram to disk.
 *
 * @param svgContent - The rendered SVG string from the WASM renderer.
 * @param format     - Target format (`'svg'` or `'png'`).
 *
 * Currently only SVG is supported (PNG conversion via `resvg` requires a
 * native binary and will be wired up in milestone Mc).
 */
export async function exportDiagram(
    svgContent: string,
    format: ExportFormat,
): Promise<void> {
    if (format === 'png') {
        vscode.window.showInformationMessage(
            'PNG export will be available in a future release. ' +
            'Save the preview as SVG for now.',
        );
        return;
    }

    const uri = await vscode.window.showSaveDialog({
        filters: { 'SVG Image': ['svg'] },
        saveLabel: 'Export SVG',
    });
    if (!uri) {
        return; // user cancelled
    }

    const encoder = new TextEncoder();
    await vscode.workspace.fs.writeFile(uri, encoder.encode(svgContent));
    vscode.window.showInformationMessage(`Diagram exported to ${uri.fsPath}`);
}
