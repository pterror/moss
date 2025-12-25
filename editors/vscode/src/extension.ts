import * as vscode from 'vscode';
import { MossRunner } from './runner';
import { MossDiagnostics } from './diagnostics';
import { SkeletonViewProvider } from './skeleton';

let runner: MossRunner;
let diagnostics: MossDiagnostics;
let skeletonProvider: SkeletonViewProvider;

export function activate(context: vscode.ExtensionContext) {
    console.log('Moss extension activated');

    // Initialize components
    runner = new MossRunner();
    diagnostics = new MossDiagnostics();
    skeletonProvider = new SkeletonViewProvider();

    // Register diagnostics collection
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('moss');
    context.subscriptions.push(diagnosticCollection);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('moss.lint', async (uri?: vscode.Uri) => {
            const targetPath = uri?.fsPath ?? vscode.window.activeTextEditor?.document.uri.fsPath ?? vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (!targetPath) {
                vscode.window.showErrorMessage('No file or workspace folder open');
                return;
            }

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: 'Running Moss lint...',
                    cancellable: false
                },
                async () => {
                    try {
                        const results = await runner.runLint(targetPath);
                        diagnostics.updateDiagnosticsFromLint(diagnosticCollection, results);

                        const totalDiagnostics = results.reduce((sum, r) => sum + r.diagnostics.length, 0);
                        const tools = results.filter(r => r.success).map(r => r.tool).join(', ');

                        if (totalDiagnostics === 0) {
                            vscode.window.showInformationMessage(`Moss: No issues found (${tools})`);
                        } else {
                            vscode.window.showWarningMessage(
                                `Moss: Found ${totalDiagnostics} issue(s) from ${tools}`
                            );
                        }
                    } catch (error) {
                        vscode.window.showErrorMessage(`Moss error: ${error}`);
                    }
                }
            );
        }),

        vscode.commands.registerCommand('moss.lintFix', async (uri?: vscode.Uri) => {
            const targetPath = uri?.fsPath ?? vscode.window.activeTextEditor?.document.uri.fsPath ?? vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (!targetPath) {
                vscode.window.showErrorMessage('No file or workspace folder open');
                return;
            }

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: 'Running Moss lint with auto-fix...',
                    cancellable: false
                },
                async () => {
                    try {
                        const results = await runner.runLint(targetPath, true);
                        diagnostics.updateDiagnosticsFromLint(diagnosticCollection, results);

                        const remainingDiagnostics = results.reduce((sum, r) => sum + r.diagnostics.length, 0);
                        if (remainingDiagnostics === 0) {
                            vscode.window.showInformationMessage('Moss: All issues fixed');
                        } else {
                            vscode.window.showWarningMessage(
                                `Moss: ${remainingDiagnostics} unfixable issue(s) remaining`
                            );
                        }
                    } catch (error) {
                        vscode.window.showErrorMessage(`Moss error: ${error}`);
                    }
                }
            );
        }),

        vscode.commands.registerCommand('moss.showSkeleton', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) {
                vscode.window.showErrorMessage('No active editor');
                return;
            }

            try {
                const skeleton = await runner.runSkeleton(editor.document.uri.fsPath);
                skeletonProvider.showSkeleton(skeleton, editor.document.fileName);
            } catch (error) {
                vscode.window.showErrorMessage(`Moss error: ${error}`);
            }
        }),

        vscode.commands.registerCommand('moss.viewTree', async (uri?: vscode.Uri) => {
            const targetPath = uri?.fsPath ?? vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (!targetPath) {
                vscode.window.showErrorMessage('No workspace folder open');
                return;
            }

            try {
                const tree = await runner.runViewTree(targetPath);
                const doc = await vscode.workspace.openTextDocument({
                    content: tree,
                    language: 'plaintext'
                });
                await vscode.window.showTextDocument(doc);
            } catch (error) {
                vscode.window.showErrorMessage(`Moss error: ${error}`);
            }
        }),

        vscode.commands.registerCommand('moss.analyzeHealth', async () => {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder open');
                return;
            }

            await vscode.window.withProgress(
                {
                    location: vscode.ProgressLocation.Notification,
                    title: 'Analyzing codebase health...',
                    cancellable: false
                },
                async () => {
                    try {
                        const health = await runner.runAnalyzeHealth(workspaceFolder.uri.fsPath);
                        const doc = await vscode.workspace.openTextDocument({
                            content: health,
                            language: 'json'
                        });
                        await vscode.window.showTextDocument(doc);
                    } catch (error) {
                        vscode.window.showErrorMessage(`Moss error: ${error}`);
                    }
                }
            );
        })
    );

    // Set up on-save diagnostics if enabled
    const config = vscode.workspace.getConfiguration('moss');
    if (config.get<boolean>('runOnSave')) {
        context.subscriptions.push(
            vscode.workspace.onDidSaveTextDocument(async (document) => {
                const supportedLanguages = ['python', 'typescript', 'typescriptreact', 'javascript', 'javascriptreact', 'rust', 'go'];
                if (supportedLanguages.includes(document.languageId)) {
                    try {
                        const results = await runner.runLint(document.uri.fsPath);
                        diagnostics.updateDiagnosticsFromLint(diagnosticCollection, results);
                    } catch {
                        // Silently ignore errors on auto-run
                    }
                }
            })
        );
    }

    // Watch for configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration('moss.binaryPath')) {
                runner.updateBinaryPath();
            }
        })
    );
}

export function deactivate() {
    console.log('Moss extension deactivated');
}
