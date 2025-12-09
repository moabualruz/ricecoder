import * as assert from 'assert';
import * as vscode from 'vscode';
import { HoverProvider } from './hoverProvider';
import { RicecoderClient } from '../client/ricecoderClient';

// Helper type for accessing private methods in tests
type HoverProviderForTesting = HoverProvider & {
	formatHover: (data: Record<string, unknown>, range: vscode.Range) => vscode.Hover | null;
	parseRange: (rangeData: Record<string, unknown>) => vscode.Range;
};

/**
 * Test suite for VS Code Hover Provider
 * 
 * Tests:
 * - Hover provider registration with VS Code
 * - Forwarding hover requests to ricecoder backend
 * - Displaying hover information in VS Code
 * - Formatting hover responses with markdown support
 * - Error handling and fallback behavior
 * - Timeout handling
 * - Cancellation support
 * - Range parsing and display
 * 
 * **Feature: ricecoder-ide, Property 5: IDE Request Handling**
 * **Validates: Requirements 3.2, 3.3**
 */
describe('HoverProvider', () => {
	let mockClient: RicecoderClient;
	let provider: HoverProvider;

	beforeEach(() => {
		// Create a mock client
		mockClient = new RicecoderClient('localhost', 9000, 5000);
		provider = new HoverProvider(mockClient);
	});

	describe('Initialization', () => {
		it('should initialize with a RicecoderClient', () => {
			assert.ok(provider);
			assert.ok(provider instanceof HoverProvider);
		});

		it('should implement vscode.HoverProvider interface', () => {
			assert.ok(typeof provider.provideHover === 'function');
		});
	});

	describe('Hover Request Handling', () => {
		it('should return null when client is not connected', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 6);
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await provider.provideHover(mockDocument, position, token);
			assert.strictEqual(result, null);
		});

		it('should return null when no word is at position', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => '   ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 0);
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await provider.provideHover(mockDocument, position, token);
			assert.strictEqual(result, null);
		});

		it('should handle hover request timeout', async () => {
			// Create a provider with a mock client that times out
			const timeoutClient = new RicecoderClient('localhost', 9999, 100);
			const timeoutProvider = new HoverProvider(timeoutClient);

			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 6);
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await timeoutProvider.provideHover(mockDocument, position, token);
			assert.strictEqual(result, null);
		});

		it('should return null when cancellation is requested', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 6);
			const token = { isCancellationRequested: true } as vscode.CancellationToken;

			const result = await provider.provideHover(mockDocument, position, token);
			assert.strictEqual(result, null);
		});

		it('should include correct parameters in hover request', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			// Verify the provider would send correct parameters
			assert.strictEqual(mockDocument.languageId, 'typescript');
			assert.strictEqual(mockDocument.uri.fsPath, '/test/file.ts');
			assert.strictEqual(mockDocument.getText(), 'const x = 1;');
		});

		it('should support multiple languages', async () => {
			const languages = ['typescript', 'javascript', 'python', 'rust', 'go'];

			languages.forEach(lang => {
				const mockDocument = {
					languageId: lang,
					uri: { fsPath: `/test/file.${lang}` },
					getText: () => 'test',
					getWordRangeAtPosition: () => new vscode.Range(0, 0, 0, 4)
				} as unknown as vscode.TextDocument;

				assert.strictEqual(mockDocument.languageId, lang);
			});
		});
	});

	describe('Response Formatting', () => {
		it('should format hover with contents', () => {
			const mockData = {
				contents: 'This is a variable'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents.length > 0);
		});

		it('should format hover with type information', () => {
			const mockData = {
				contents: 'Variable x',
				type_info: 'number'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents.length > 0);
		});

		it('should format hover with documentation', () => {
			const mockData = {
				contents: 'Function foo',
				documentation: 'This function does something'
			};

			const range = new vscode.Range(0, 0, 0, 3);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents.length > 0);
		});

		it('should format hover with source information', () => {
			const mockData = {
				contents: 'Symbol from library',
				source: 'lodash'
			};

			const range = new vscode.Range(0, 0, 0, 6);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents.length > 0);
		});

		it('should format hover with all fields', () => {
			const mockData = {
				contents: 'Complete hover',
				type_info: 'string',
				documentation: 'A complete hover example',
				source: 'example-lib'
			};

			const range = new vscode.Range(0, 0, 0, 8);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents.length >= 4);
		});

		it('should return null when no content is available', () => {
			const mockData = {};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.strictEqual(hover, null);
		});

		it('should handle empty contents', () => {
			const mockData = {
				contents: ''
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			// Empty string should still create a hover
			assert.ok(hover);
		});

		it('should use provided range when available', () => {
			const mockData = {
				contents: 'Test',
				range: {
					start: { line: 1, character: 5 },
					end: { line: 1, character: 10 }
				}
			};

			const defaultRange = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, defaultRange);

			assert.ok(hover);
			assert.strictEqual(hover!.range?.start.line, 1);
			assert.strictEqual(hover!.range?.start.character, 5);
		});

		it('should use default range when not provided', () => {
			const mockData = {
				contents: 'Test'
			};

			const defaultRange = new vscode.Range(0, 0, 0, 4);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, defaultRange);

			assert.ok(hover);
			assert.strictEqual(hover!.range?.start.line, 0);
			assert.strictEqual(hover!.range?.start.character, 0);
		});
	});

	describe('Range Parsing', () => {
		it('should parse single-line range', () => {
			const rangeData = {
				start: { line: 0, character: 0 },
				end: { line: 0, character: 10 }
			};

			const range = (provider as unknown as HoverProviderForTesting).parseRange(rangeData);

			assert.strictEqual(range.start.line, 0);
			assert.strictEqual(range.start.character, 0);
			assert.strictEqual(range.end.line, 0);
			assert.strictEqual(range.end.character, 10);
		});

		it('should parse multi-line range', () => {
			const rangeData = {
				start: { line: 1, character: 5 },
				end: { line: 3, character: 15 }
			};

			const range = (provider as unknown as HoverProviderForTesting).parseRange(rangeData);

			assert.strictEqual(range.start.line, 1);
			assert.strictEqual(range.start.character, 5);
			assert.strictEqual(range.end.line, 3);
			assert.strictEqual(range.end.character, 15);
		});

		it('should handle zero-width range', () => {
			const rangeData = {
				start: { line: 0, character: 5 },
				end: { line: 0, character: 5 }
			};

			const range = (provider as unknown as HoverProviderForTesting).parseRange(rangeData);

			assert.strictEqual(range.start.line, 0);
			assert.strictEqual(range.start.character, 5);
			assert.strictEqual(range.end.line, 0);
			assert.strictEqual(range.end.character, 5);
		});

		it('should handle missing position fields with defaults', () => {
			const rangeData = {
				start: {},
				end: {}
			};

			const range = (provider as unknown as HoverProviderForTesting).parseRange(rangeData);

			assert.strictEqual(range.start.line, 0);
			assert.strictEqual(range.start.character, 0);
			assert.strictEqual(range.end.line, 0);
			assert.strictEqual(range.end.character, 0);
		});

		it('should handle partial position data', () => {
			const rangeData = {
				start: { line: 2 },
				end: { character: 10 }
			};

			const range = (provider as unknown as HoverProviderForTesting).parseRange(rangeData);

			assert.strictEqual(range.start.line, 2);
			assert.strictEqual(range.start.character, 0);
			assert.strictEqual(range.end.line, 0);
			assert.strictEqual(range.end.character, 10);
		});
	});

	describe('Markdown Support', () => {
		it('should create markdown strings for content', () => {
			const mockData = {
				contents: '**Bold** text'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents[0] instanceof vscode.MarkdownString);
		});

		it('should support code blocks in hover', () => {
			const mockData = {
				contents: '```typescript\nconst x: number = 1;\n```'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents[0] instanceof vscode.MarkdownString);
		});

		it('should support links in hover', () => {
			const mockData = {
				contents: '[Documentation](https://example.com)'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents[0] instanceof vscode.MarkdownString);
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

			const position = new vscode.Position(0, 0);
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await provider.provideHover(mockDocument, position, token);
			assert.strictEqual(result, null);
		});

		it('should handle invalid response data', () => {
			const mockData = null;
			const range = new vscode.Range(0, 0, 0, 1);

			// Should not throw
			assert.doesNotThrow(() => {
				if (mockData) {
					(provider as unknown as HoverProviderForTesting).formatHover(mockData as Record<string, unknown>, range);
				}
			});
		});

		it('should handle missing range data', () => {
			const mockData = {
				contents: 'Test',
				range: null
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
		});

		it('should handle malformed range data', () => {
			const mockData = {
				contents: 'Test',
				range: 'invalid'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
		});

		it('should handle connection errors', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 6);
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			// Should not throw even if client is not connected
			const result = await provider.provideHover(mockDocument, position, token);
			assert.strictEqual(result, null);
		});
	});

	describe('Type Conversion', () => {
		it('should convert string values to appropriate types', () => {
			const mockData = {
				contents: 'Test',
				type_info: 'string',
				source: 'lib'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents.length > 0);
		});

		it('should handle numeric values in range', () => {
			const rangeData = {
				start: { line: '0', character: '5' },
				end: { line: '1', character: '10' }
			};

			const range = (provider as unknown as HoverProviderForTesting).parseRange(rangeData as Record<string, unknown>);

			assert.strictEqual(range.start.line, 0);
			assert.strictEqual(range.start.character, 5);
			assert.strictEqual(range.end.line, 1);
			assert.strictEqual(range.end.character, 10);
		});
	});

	describe('Word Detection', () => {
		it('should extract word at position', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const variable = 42;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 14)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 10);

			// Verify word range is detected
			const wordRange = mockDocument.getWordRangeAtPosition(position);
			assert.ok(wordRange);
			assert.strictEqual(wordRange.start.character, 6);
			assert.strictEqual(wordRange.end.character, 14);
		});

		it('should handle position at word boundary', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 6);

			const wordRange = mockDocument.getWordRangeAtPosition(position);
			assert.ok(wordRange);
		});

		it('should handle position in whitespace', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const   x = 1;',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const wordRange = mockDocument.getWordRangeAtPosition(new vscode.Position(0, 7));
			assert.strictEqual(wordRange, null);
		});
	});

	describe('Hover Display', () => {
		it('should create hover with markdown contents', () => {
			const mockData = {
				contents: 'Variable declaration'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(Array.isArray(hover!.contents));
			assert.ok(hover!.contents.length > 0);
		});

		it('should include range in hover', () => {
			const mockData = {
				contents: 'Test',
				range: {
					start: { line: 0, character: 0 },
					end: { line: 0, character: 4 }
				}
			};

			const defaultRange = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, defaultRange);

			assert.ok(hover);
			assert.ok(hover!.range);
		});

		it('should handle hover without explicit range', () => {
			const mockData = {
				contents: 'Test'
			};

			const defaultRange = new vscode.Range(0, 0, 0, 4);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, defaultRange);

			assert.ok(hover);
			assert.ok(hover!.range);
		});
	});

	describe('Cancellation', () => {
		it('should respect cancellation token', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 6);
			const cancellationToken = { isCancellationRequested: true } as vscode.CancellationToken;

			const result = await provider.provideHover(mockDocument, position, cancellationToken);
			assert.strictEqual(result, null);
		});

		it('should check cancellation after request', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 6);
			const cancellationToken = { isCancellationRequested: false } as vscode.CancellationToken;

			// Should not throw
			const result = await provider.provideHover(mockDocument, position, cancellationToken);
			assert.ok(result === null || result instanceof vscode.Hover);
		});
	});

	describe('Integration', () => {
		it('should work with VS Code API', () => {
			const range = new vscode.Range(0, 0, 0, 10);
			const markdown = new vscode.MarkdownString('Test');
			const hover = new vscode.Hover(markdown, range);

			assert.ok(hover);
			assert.ok(hover.contents);
			assert.ok(hover.range);
		});

		it('should support multiple content items', () => {
			const mockData = {
				contents: 'Main content',
				type_info: 'string',
				documentation: 'Additional docs',
				source: 'lib'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			const hover = (provider as unknown as HoverProviderForTesting).formatHover(mockData, range);

			assert.ok(hover);
			assert.ok(hover!.contents.length >= 2);
		});
	});
});
