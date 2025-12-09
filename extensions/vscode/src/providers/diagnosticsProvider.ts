import * as vscode from 'vscode';
import { RicecoderClient } from '../client/ricecoderClient';

/**
 * Diagnostics provider for VS Code
 * 
 * Manages diagnostics by:
 * 1. Monitoring open documents for changes
 * 2. Requesting diagnostics from RiceCoder backend
 * 3. Displaying diagnostics in VS Code editor
 * 4. Supporting quick fixes from diagnostics
 */
export class DiagnosticsProvider implements vscode.Disposable {
	private diagnosticCollection: vscode.DiagnosticCollection;
	private disposables: vscode.Disposable[] = [];
	private documentTimers: Map<string, NodeJS.Timeout> = new Map();
	private debounceDelay: number = 500; // ms

	constructor(private client: RicecoderClient) {
		this.diagnosticCollection = vscode.languages.createDiagnosticCollection('ricecoder');
	}

	/**
	 * Start monitoring documents for diagnostics
	 */
	start(): void {
		// Monitor document changes
		const changeDisposable = vscode.workspace.onDidChangeTextDocument((event) => {
			this.handleDocumentChange(event.document);
		});
		this.disposables.push(changeDisposable);

		// Monitor document saves
		const saveDisposable = vscode.workspace.onDidSaveTextDocument((document) => {
			this.updateDiagnostics(document);
		});
		this.disposables.push(saveDisposable);

		// Monitor document opens
		const openDisposable = vscode.workspace.onDidOpenTextDocument((document) => {
			this.updateDiagnostics(document);
		});
		this.disposables.push(openDisposable);

		// Monitor document closes
		const closeDisposable = vscode.workspace.onDidCloseTextDocument((document) => {
			this.diagnosticCollection.delete(document.uri);
			this.documentTimers.delete(document.uri.fsPath);
		});
		this.disposables.push(closeDisposable);

		// Process currently open documents
		for (const document of vscode.workspace.textDocuments) {
			this.updateDiagnostics(document);
		}
	}

	/**
	 * Stop monitoring documents
	 */
	stop(): void {
		// Clear all timers
		for (const timer of this.documentTimers.values()) {
			clearTimeout(timer);
		}
		this.documentTimers.clear();

		// Dispose all listeners
		for (const disposable of this.disposables) {
			disposable.dispose();
		}
		this.disposables = [];
	}

	/**
	 * Handle document changes with debouncing
	 */
	private handleDocumentChange(document: vscode.TextDocument): void {
		// Clear existing timer for this document
		const key = document.uri.fsPath;
		const existingTimer = this.documentTimers.get(key);
		if (existingTimer) {
			clearTimeout(existingTimer);
		}

		// Set new debounced timer
		const timer = setTimeout(() => {
			this.updateDiagnostics(document);
			this.documentTimers.delete(key);
		}, this.debounceDelay);

		this.documentTimers.set(key, timer);
	}

	/**
	 * Update diagnostics for a document
	 */
	private async updateDiagnostics(document: vscode.TextDocument): Promise<void> {
		try {
			if (!this.client.isConnected()) {
				return;
			}

			// Skip unsupported document types
			if (document.uri.scheme !== 'file') {
				return;
			}

			// Request diagnostics from backend
			const params = {
				language: document.languageId,
				file_path: document.uri.fsPath,
				source: document.getText()
			};

			const result = await Promise.race([
				this.client.request('diagnostics/provide', params),
				new Promise((_, reject) =>
					setTimeout(() => reject(new Error('Diagnostics request timeout')), 5000)
				)
			]);

			// Format and display diagnostics
			const diagnostics = this.formatDiagnostics(result as unknown[]);
			this.diagnosticCollection.set(document.uri, diagnostics);
		} catch (error) {
			console.error('Diagnostics provider error:', error);
		}
	}

	/**
	 * Format diagnostics from backend response
	 */
	private formatDiagnostics(items: unknown[]): vscode.Diagnostic[] {
		return items.map((item: unknown) => {
			const data = item as Record<string, unknown>;
			const range = data.range as Record<string, unknown>;
			const start = range.start as Record<string, unknown>;
			const end = range.end as Record<string, unknown>;

			const diagnostic = new vscode.Diagnostic(
				new vscode.Range(
					Number(start.line || 0),
					Number(start.character || 0),
					Number(end.line || 0),
					Number(end.character || 0)
				),
				String(data.message || ''),
				this.mapSeverity(String(data.severity || 'Error'))
			);

			diagnostic.source = data.source ? String(data.source) : 'RiceCoder';
			diagnostic.code = data.code ? String(data.code) : undefined;

			// Add quick fixes if available
			if (data.quick_fixes && Array.isArray(data.quick_fixes)) {
				const fixes = data.quick_fixes as unknown[];
				diagnostic.relatedInformation = fixes.map((fix: unknown) => {
					const fixData = fix as Record<string, unknown>;
					return new vscode.DiagnosticRelatedInformation(
						new vscode.Location(
							vscode.Uri.file(String(fixData.file || '')),
							new vscode.Position(
								Number(fixData.line || 0),
								Number(fixData.character || 0)
							)
						),
						String(fixData.message || '')
					);
				});
			}

			return diagnostic;
		});
	}

	/**
	 * Map backend severity to VS Code severity
	 */
	private mapSeverity(severity: string): vscode.DiagnosticSeverity {
		const severityMap: Record<string, vscode.DiagnosticSeverity> = {
			'Error': vscode.DiagnosticSeverity.Error,
			'Warning': vscode.DiagnosticSeverity.Warning,
			'Information': vscode.DiagnosticSeverity.Information,
			'Hint': vscode.DiagnosticSeverity.Hint
		};

		return severityMap[severity] || vscode.DiagnosticSeverity.Error;
	}

	/**
	 * Dispose of resources
	 */
	dispose(): void {
		this.stop();
		this.diagnosticCollection.dispose();
	}
}
