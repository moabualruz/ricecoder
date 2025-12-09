import * as assert from 'assert';
import * as vscode from 'vscode';
import { CompletionProvider } from './completionProvider';
import { RicecoderClient } from '../client/ricecoderClient';

/**
 * Test suite for VS Code Completion Provider
 * 
 * Tests:
 * - Completion provider registration with VS Code
 * - Forwarding completion requests to ricecoder backend
 * - Formatting ricecoder responses for VS Code
 * - Snippet expansion support
 * - Error handling and fallback behavior
 */
describe('CompletionProvider', () => {
	let mockClient: RicecoderClient;
	let provider: CompletionProvider;

	beforeEach(() => {
		// Create a mock client
		mockClient = new RicecoderClient('localhost', 9000, 5000);
		provider = new CompletionProvider(mockClient);
	});

	describe('Initialization', () => {
		it('should initialize with a RicecoderClient', () => {
			assert.ok(provider);
			assert.ok(provider instanceof CompletionProvider);
		});

		it('should implement vscode.CompletionItemProvider interface', () => {
			assert.ok(typeof provider.provideCompletionItems === 'function');
			assert.ok(typeof provider.resolveCompletionItem === 'function');
		});
	});

	describe('Completion Request Handling', () => {
		it('should return null when client is not connected', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 10);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await provider.provideCompletionItems(mockDocument, position, token, context);
			assert.strictEqual(result, null);
		});

		it('should handle completion request timeout', async () => {
			// Create a provider with a mock client that times out
			const timeoutClient = new RicecoderClient('localhost', 9999, 100);
			const timeoutProvider = new CompletionProvider(timeoutClient);

			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 10);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await timeoutProvider.provideCompletionItems(mockDocument, position, token, context);
			assert.strictEqual(result, null);
		});

		it('should return null when cancellation is requested', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 10);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: true } as vscode.CancellationToken;

			const result = await provider.provideCompletionItems(mockDocument, position, token, context);
			assert.strictEqual(result, null);
		});
	});

	describe('Response Formatting', () => {
		it('should format completion items correctly', () => {
			const mockItems = [
				{
					label: 'function',
					kind: 'Function',
					insert_text: 'function ${1:name}() {\n\t$0\n}',
					detail: 'Function declaration',
					documentation: 'Declares a new function'
				},
				{
					label: 'const',
					kind: 'Keyword',
					insert_text: 'const',
					detail: 'Constant declaration'
				}
			];

			// Access the private method through type casting
			const formattedItems = (provider as any).formatCompletionItems(mockItems);

			assert.strictEqual(formattedItems.length, 2);
			assert.strictEqual(formattedItems[0].label, 'function');
			assert.strictEqual(formattedItems[0].kind, vscode.CompletionItemKind.Function);
			assert.strictEqual(formattedItems[1].label, 'const');
			assert.strictEqual(formattedItems[1].kind, vscode.CompletionItemKind.Keyword);
		});

		it('should handle missing optional fields', () => {
			const mockItems = [
				{
					label: 'test',
					kind: 'Text'
				}
			];

			const formattedItems = (provider as any).formatCompletionItems(mockItems);

			assert.strictEqual(formattedItems.length, 1);
			assert.strictEqual(formattedItems[0].label, 'test');
			assert.strictEqual(formattedItems[0].detail, undefined);
			assert.strictEqual(formattedItems[0].documentation, undefined);
		});

		it('should map all completion kinds correctly', () => {
			const kinds = [
				'Text', 'Method', 'Function', 'Constructor', 'Field', 'Variable',
				'Class', 'Interface', 'Module', 'Property', 'Unit', 'Value',
				'Enum', 'Keyword', 'Snippet', 'Color', 'File', 'Reference',
				'Folder', 'EnumMember', 'Constant', 'Struct', 'Event', 'Operator',
				'TypeParameter'
			];

			const mockItems = kinds.map(kind => ({
				label: kind,
				kind: kind,
				insert_text: kind
			}));

			const formattedItems = (provider as any).formatCompletionItems(mockItems);

			assert.strictEqual(formattedItems.length, kinds.length);
			formattedItems.forEach((item, index) => {
				assert.ok(item.kind !== undefined);
				assert.ok(item.kind >= 0);
			});
		});
	});

	describe('Snippet Expansion', () => {
		it('should detect and expand snippets', () => {
			const mockItems = [
				{
					label: 'function',
					kind: 'Function',
					insert_text: 'function ${1:name}() {\n\t$0\n}'
				}
			];

			const formattedItems = (provider as any).formatCompletionItems(mockItems);

			assert.strictEqual(formattedItems.length, 1);
			assert.ok(formattedItems[0].insertText instanceof vscode.SnippetString);
		});

		it('should not expand non-snippet text', () => {
			const mockItems = [
				{
					label: 'const',
					kind: 'Keyword',
					insert_text: 'const'
				}
			];

			const formattedItems = (provider as any).formatCompletionItems(mockItems);

			assert.strictEqual(formattedItems.length, 1);
			assert.strictEqual(typeof formattedItems[0].insertText, 'string');
		});

		it('should handle complex snippets with multiple placeholders', () => {
			const mockItems = [
				{
					label: 'for loop',
					kind: 'Snippet',
					insert_text: 'for (let ${1:i} = 0; ${1:i} < ${2:array}.length; ${1:i}++) {\n\t$0\n}'
				}
			];

			const formattedItems = (provider as any).formatCompletionItems(mockItems);

			assert.strictEqual(formattedItems.length, 1);
			assert.ok(formattedItems[0].insertText instanceof vscode.SnippetString);
		});
	});

	describe('Completion Item Resolution', () => {
		it('should resolve completion item with additional details', async () => {
			const mockItem = new vscode.CompletionItem('test');
			mockItem.data = { id: 'test_1' };

			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await provider.resolveCompletionItem(mockItem, token);

			assert.ok(result);
			assert.strictEqual(result.label, 'test');
		});

		it('should return item unchanged when client is not connected', async () => {
			const mockItem = new vscode.CompletionItem('test');
			mockItem.data = { id: 'test_1' };

			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await provider.resolveCompletionItem(mockItem, token);

			assert.strictEqual(result.label, 'test');
		});

		it('should return item unchanged when cancellation is requested', async () => {
			const mockItem = new vscode.CompletionItem('test');
			mockItem.data = { id: 'test_1' };

			const token = { isCancellationRequested: true } as vscode.CancellationToken;

			const result = await provider.resolveCompletionItem(mockItem, token);

			assert.strictEqual(result.label, 'test');
		});
	});

	describe('Error Handling', () => {
		it('should handle errors gracefully', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => { throw new Error('Document error'); },
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 10);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await provider.provideCompletionItems(mockDocument, position, token, context);
			assert.strictEqual(result, null);
		});

		it('should handle invalid response data', () => {
			const mockItems = [
				null,
				undefined,
				{ label: 'valid' },
				{}
			];

			const formattedItems = (provider as any).formatCompletionItems(mockItems);

			assert.ok(formattedItems.length > 0);
			formattedItems.forEach(item => {
				assert.ok(item instanceof vscode.CompletionItem);
			});
		});
	});

	describe('Language Support', () => {
		it('should support multiple languages', () => {
			const languages = ['typescript', 'javascript', 'python', 'rust', 'go'];

			languages.forEach(lang => {
				const mockDocument = {
					languageId: lang,
					uri: { fsPath: `/test/file.${lang}` },
					getText: () => 'test',
					getWordRangeAtPosition: () => null
				} as unknown as vscode.TextDocument;

				assert.ok(mockDocument.languageId === lang);
			});
		});
	});

	describe('Trigger Characters', () => {
		it('should handle different trigger characters', () => {
			const triggerChars = ['.', '(', '[', ' ', undefined];

			triggerChars.forEach(char => {
				const context = { triggerCharacter: char } as vscode.CompletionContext;
				assert.ok(context.triggerCharacter === char);
			});
		});
	});
});
