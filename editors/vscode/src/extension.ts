import * as vscode from 'vscode';
import { PreviewPanel } from './preview';

/**
 * Called when the extension is first activated (on opening a .mmd/.mermaid file).
 */
export function activate(context: vscode.ExtensionContext): void {
    // Register "Open Preview" command
    context.subscriptions.push(
        vscode.commands.registerCommand('trellis.openPreview', () => {
            openPreviewForActiveEditor(context);
        }),
    );

    // Register "Open Preview to the Side" command
    context.subscriptions.push(
        vscode.commands.registerCommand('trellis.openPreviewToSide', () => {
            openPreviewForActiveEditor(context, vscode.ViewColumn.Beside);
        }),
    );

    // Auto-update the preview when the document content changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeTextDocument((event) => {
            if (isMermaidDocument(event.document)) {
                const cfg = vscode.workspace.getConfiguration('trellis');
                if (cfg.get<boolean>('autoPreview', true)) {
                    PreviewPanel.update(event.document.getText());
                }
            }
        }),
    );

    // Also update when the active editor switches to a .mmd file
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor((editor) => {
            if (editor && isMermaidDocument(editor.document)) {
                PreviewPanel.update(editor.document.getText());
            }
        }),
    );
}

/** Called when the extension is deactivated. */
export function deactivate(): void {
    PreviewPanel.dispose();
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function openPreviewForActiveEditor(
    context: vscode.ExtensionContext,
    column: vscode.ViewColumn = vscode.ViewColumn.Active,
): void {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('Trellis: No active editor found.');
        return;
    }
    if (!isMermaidDocument(editor.document)) {
        vscode.window.showWarningMessage(
            'Trellis: Open a .mmd or .mermaid file to preview it.',
        );
        return;
    }
    PreviewPanel.createOrShow(context.extensionUri, editor.document.getText(), column);
}

function isMermaidDocument(document: vscode.TextDocument): boolean {
    return (
        document.languageId === 'mermaid' ||
        document.fileName.endsWith('.mmd') ||
        document.fileName.endsWith('.mermaid')
    );
}
