import * as vscode from 'vscode';
import { RicecoderClient } from '../client/ricecoderClient';

/**
 * Completion provider for VS Code
 * 
 * Registers with VS Code to provide code completions by:
 * 1. Forwarding completion requests to RiceCoder backend
 * 2. Formatting responses for VS Code consumption
 * 3. Supporting snippet expansion
 */
export class CompletionProvider implements vscode.CompletionItemProvider {
	constructor(private client: RicecoderClient) {}

	/**
	 * Provide completion items for the given position
	 */
	async provideCompletionItems(
		document: vscode.TextDocument,
		position: vscode.Position,
		token: vscode.CancellationToken,
		context: vscode.CompletionContext
	): Promise<vscode.CompletionItem[] | vscode.CompletionList | null> {
		try {
			if (!this.client.isConnected()) {
				return null;
			}

			// Get context around the cursor
			const lineStart = new vscode.Position(position.line, 0);
			const lineEnd = new vscode.Position(position.line, position.character);
			const lineContext = document.getText(new vscode.Range(lineStart, lineEnd));

			// Prepare completion request
			const params = {
				language: document.languageId,
				file_path: document.uri.fsPath,
				position: {
					line: position.line,
					character: position.character
				},
				context: lineContext,
				trigger_character: context.triggerCharacter
			};

			// Request completions from backend
			const result = await Promise.race([
				this.client.request('completion/provide', params),
				new Promise((_, reject) =>
					setTimeout(() => reject(new Error('Completion request timeout')), 5000)
				)
			]);

			if (token.isCancellationRequested) {
				return null;
			}

			// Format response for VS Code
			return this.formatCompletionItems(result as unknown[]);
		} catch (error) {
			console.error('Completion provider error:', error);
			return null;
		}
	}

	/**
	 * Resolve a completion item (provide additional details)
	 */
	async resolveCompletionItem(
		item: vscode.CompletionItem,
		token: vscode.CancellationToken
	): Promise<vscode.CompletionItem> {
		try {
			const itemWithData = item as vscode.CompletionItem & { data?: unknown };
			if (!this.client.isConnected() || !itemWithData.data) {
				return item;
			}

			// Request additional details from backend
			const result = await this.client.request('completion/resolve', itemWithData.data);

			if (token.isCancellationRequested) {
				return item;
			}

			// Update item with resolved details
			const resolved = result as Record<string, unknown>;
			if (resolved.documentation) {
				item.documentation = new vscode.MarkdownString(String(resolved.documentation));
			}
			if (resolved.detail) {
				item.detail = String(resolved.detail);
			}

			return item;
		} catch (error) {
			console.error('Completion resolve error:', error);
			return item;
		}
	}

	/**
	 * Format completion items from backend response
	 */
	private formatCompletionItems(items: unknown[]): vscode.CompletionItem[] {
		return items.map((item: unknown) => {
			const data = item as Record<string, unknown>;
			const completionItem = new vscode.CompletionItem(
				String(data.label || ''),
				this.mapCompletionKind(String(data.kind || 'Text'))
			);

			completionItem.insertText = String(data.insert_text || data.label || '');
			completionItem.detail = data.detail ? String(data.detail) : undefined;
			completionItem.documentation = data.documentation
				? new vscode.MarkdownString(String(data.documentation))
				: undefined;
			completionItem.sortText = data.sort_text ? String(data.sort_text) : undefined;
			completionItem.filterText = data.filter_text ? String(data.filter_text) : undefined;
			(completionItem as vscode.CompletionItem & { data?: unknown }).data = data;

			// Handle snippet expansion
			if (data.insert_text && String(data.insert_text).includes('$')) {
				completionItem.insertText = new vscode.SnippetString(String(data.insert_text));
			}

			return completionItem;
		});
	}

	/**
	 * Map backend completion kind to VS Code completion kind
	 */
	private mapCompletionKind(kind: string): vscode.CompletionItemKind {
		const kindMap: Record<string, vscode.CompletionItemKind> = {
			'Text': vscode.CompletionItemKind.Text,
			'Method': vscode.CompletionItemKind.Method,
			'Function': vscode.CompletionItemKind.Function,
			'Constructor': vscode.CompletionItemKind.Constructor,
			'Field': vscode.CompletionItemKind.Field,
			'Variable': vscode.CompletionItemKind.Variable,
			'Class': vscode.CompletionItemKind.Class,
			'Interface': vscode.CompletionItemKind.Interface,
			'Module': vscode.CompletionItemKind.Module,
			'Property': vscode.CompletionItemKind.Property,
			'Unit': vscode.CompletionItemKind.Unit,
			'Value': vscode.CompletionItemKind.Value,
			'Enum': vscode.CompletionItemKind.Enum,
			'Keyword': vscode.CompletionItemKind.Keyword,
			'Snippet': vscode.CompletionItemKind.Snippet,
			'Color': vscode.CompletionItemKind.Color,
			'File': vscode.CompletionItemKind.File,
			'Reference': vscode.CompletionItemKind.Reference,
			'Folder': vscode.CompletionItemKind.Folder,
			'EnumMember': vscode.CompletionItemKind.EnumMember,
			'Constant': vscode.CompletionItemKind.Constant,
			'Struct': vscode.CompletionItemKind.Struct,
			'Event': vscode.CompletionItemKind.Event,
			'Operator': vscode.CompletionItemKind.Operator,
			'TypeParameter': vscode.CompletionItemKind.TypeParameter
		};

		return kindMap[kind] || vscode.CompletionItemKind.Text;
	}
}
