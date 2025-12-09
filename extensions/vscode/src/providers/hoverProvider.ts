import * as vscode from 'vscode';
import { RicecoderClient } from '../client/ricecoderClient';

/**
 * Hover provider for VS Code
 * 
 * Provides hover information by:
 * 1. Forwarding hover requests to RiceCoder backend
 * 2. Formatting responses for VS Code display
 * 3. Supporting markdown formatting
 */
export class HoverProvider implements vscode.HoverProvider {
	constructor(private client: RicecoderClient) {}

	/**
	 * Provide hover information for the given position
	 */
	async provideHover(
		document: vscode.TextDocument,
		position: vscode.Position,
		token: vscode.CancellationToken
	): Promise<vscode.Hover | null> {
		try {
			if (!this.client.isConnected()) {
				return null;
			}

			// Get the word at the current position
			const wordRange = document.getWordRangeAtPosition(position);
			if (!wordRange) {
				return null;
			}

			const word = document.getText(wordRange);

			// Prepare hover request
			const params = {
				language: document.languageId,
				file_path: document.uri.fsPath,
				position: {
					line: position.line,
					character: position.character
				},
				word
			};

			// Request hover information from backend
			const result = await Promise.race([
				this.client.request('hover/provide', params),
				new Promise((_, reject) =>
					setTimeout(() => reject(new Error('Hover request timeout')), 5000)
				)
			]);

			if (token.isCancellationRequested) {
				return null;
			}

			if (!result) {
				return null;
			}

			// Format response for VS Code
			return this.formatHover(result as Record<string, unknown>, wordRange);
		} catch (error) {
			console.error('Hover provider error:', error);
			return null;
		}
	}

	/**
	 * Format hover information from backend response
	 */
	private formatHover(
		data: Record<string, unknown>,
		range: vscode.Range
	): vscode.Hover | null {
		const contents: vscode.MarkdownString[] = [];

		// Add main content
		if (data.contents) {
			const content = String(data.contents);
			contents.push(new vscode.MarkdownString(content));
		}

		// Add type information if available
		if (data.type_info) {
			const typeInfo = String(data.type_info);
			contents.push(new vscode.MarkdownString(`**Type:** \`${typeInfo}\``));
		}

		// Add documentation if available
		if (data.documentation) {
			const doc = String(data.documentation);
			contents.push(new vscode.MarkdownString(doc));
		}

		// Add source information if available
		if (data.source) {
			const source = String(data.source);
			contents.push(new vscode.MarkdownString(`**Source:** ${source}`));
		}

		if (contents.length === 0) {
			return null;
		}

		// Create hover with optional range
		const hoverRange = data.range ? this.parseRange(data.range as Record<string, unknown>) : range;
		return new vscode.Hover(contents, hoverRange);
	}

	/**
	 * Parse range from backend response
	 */
	private parseRange(rangeData: Record<string, unknown>): vscode.Range {
		const start = rangeData.start as Record<string, unknown>;
		const end = rangeData.end as Record<string, unknown>;

		return new vscode.Range(
			Number(start.line || 0),
			Number(start.character || 0),
			Number(end.line || 0),
			Number(end.character || 0)
		);
	}
}
