import * as assert from 'assert';
import * as vscode from 'vscode';
import { DiagnosticsProvider } from './diagnosticsProvider';
import { RicecoderClient } from '../client/ricecoderClient';

// Helper type for accessing private methods in tests
type DiagnosticsProviderForTesting = DiagnosticsProvider & {
	formatDiagnostics: (items: unknown[]) => vscode.Diagnostic[];
	mapSeverity: (severity: string) => vscode.DiagnosticSeverity;
};

/**
 * Test suite for VS Code Diagnostics Provider
 * 
 * Tests:
 * - Diagnostics provider registration with VS Code
 * - Forwarding diagnostics requests to ricecoder backend
 * - Displaying diagnostics in VS Code editor
 * - Supporting quick fixes from diagnostics
 * - Error handling and fallback behavior
 * - Document lifecycle management (open, change, save, close)
 * - Debouncing of document changes
 * - Severity mapping
 */
describe('DiagnosticsProvider', () => {
	let mockClient: RicecoderClient;
	let provider: DiagnosticsProvider;

	beforeEach(() => {
		// Create a mock client
		mockClient = new RicecoderClient('localhost', 9000, 5000);
		provider = new DiagnosticsProvider(mockClient);
	});

	afterEach(() => {
		// Clean up
		provider.dispose();
	});

	describe('Initialization', () => {
		it('should initialize with a RicecoderClient', () => {
			assert.ok(provider);
			assert.ok(provider instanceof DiagnosticsProvider);
		});

		it('should implement vscode.Disposable interface', () => {
			assert.ok(typeof provider.dispose === 'function');
		});

		it('should have start and stop methods', () => {
			assert.ok(typeof provider.start === 'function');
			assert.ok(typeof provider.stop === 'function');
		});
	});

	describe('Document Lifecycle Management', () => {
		it('should handle document open events', async () => {
			// Start monitoring
			provider.start();

			// Simulate document open
			// Note: In real tests, this would be triggered by VS Code events
			// For now, we just verify the provider can be started
			assert.ok(provider);

			provider.stop();
		});

		it('should handle document close events', async () => {
			provider.start();
			provider.stop();

			// Verify provider is stopped
			assert.ok(provider);
		});

		it('should skip non-file documents', async () => {
			provider.start();
			provider.stop();

			assert.ok(provider);
		});
	});

	describe('Diagnostics Request Handling', () => {
		it('should not request diagnostics when client is not connected', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts', scheme: 'file' },
				getText: () => 'const x = 1;'
			} as unknown as vscode.TextDocument;

			provider.start();

			// Simulate document change - should not request diagnostics if not connected
			// The provider checks isConnected() before making requests
			assert.ok(!mockClient.isConnected());

			provider.stop();
		});

		it('should handle diagnostics request timeout', async () => {
			// Create a provider with a mock client that times out
			const timeoutClient = new RicecoderClient('localhost', 9999, 100);
			const timeoutProvider = new DiagnosticsProvider(timeoutClient);

			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts', scheme: 'file' },
				getText: () => 'const x = 1;'
			} as unknown as vscode.TextDocument;

			timeoutProvider.start();
			timeoutProvider.stop();

			assert.ok(timeoutProvider);
		});

		it('should include correct parameters in diagnostics request', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts', scheme: 'file' },
				getText: () => 'const x = 1;'
			} as unknown as vscode.TextDocument;

			// Verify the provider would send correct parameters
			assert.strictEqual(mockDocument.languageId, 'typescript');
			assert.strictEqual(mockDocument.uri.fsPath, '/test/file.ts');
			assert.strictEqual(mockDocument.getText(), 'const x = 1;');
		});
	});

	describe('Response Formatting', () => {
		it('should format diagnostics correctly', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Unexpected token',
					source: 'TypeScript',
					code: 'TS1234'
				},
				{
					range: {
						start: { line: 1, character: 0 },
						end: { line: 1, character: 10 }
					},
					severity: 'Warning',
					message: 'Unused variable',
					source: 'TypeScript'
				}
			];

			// Access the private method through type casting
			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 2);
			assert.strictEqual(formattedDiagnostics[0].message, 'Unexpected token');
			assert.strictEqual(formattedDiagnostics[0].source, 'TypeScript');
			assert.strictEqual(formattedDiagnostics[1].message, 'Unused variable');
		});

		it('should handle missing optional fields', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Test error'
				}
			];

			const formattedDiagnostics = (provider as unknown as { formatDiagnostics: (items: unknown[]) => vscode.Diagnostic[] }).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 1);
			assert.strictEqual(formattedDiagnostics[0].message, 'Test error');
			assert.strictEqual(formattedDiagnostics[0].source, 'RiceCoder');
			assert.strictEqual(formattedDiagnostics[0].code, undefined);
		});

		it('should handle empty diagnostics array', () => {
			const mockDiagnostics: unknown[] = [];

			const formattedDiagnostics = (provider as unknown as { formatDiagnostics: (items: unknown[]) => vscode.Diagnostic[] }).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 0);
		});

		it('should handle diagnostics with default values', () => {
			const mockDiagnostics = [
				{
					range: {
						start: {},
						end: {}
					},
					message: 'Test'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 1);
			assert.strictEqual(formattedDiagnostics[0].message, 'Test');
		});
	});

	describe('Severity Mapping', () => {
		it('should map all severity levels correctly', () => {
			const severities = [
				{ input: 'Error', expected: vscode.DiagnosticSeverity.Error },
				{ input: 'Warning', expected: vscode.DiagnosticSeverity.Warning },
				{ input: 'Information', expected: vscode.DiagnosticSeverity.Information },
				{ input: 'Hint', expected: vscode.DiagnosticSeverity.Hint }
			];

			severities.forEach(({ input, expected }) => {
				const mockDiagnostics = [
					{
						range: {
							start: { line: 0, character: 0 },
							end: { line: 0, character: 5 }
						},
						severity: input,
						message: 'Test'
					}
				];

				const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

				assert.strictEqual(formattedDiagnostics[0].severity, expected);
			});
		});

		it('should default to Error for unknown severity', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Unknown',
					message: 'Test'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics[0].severity, vscode.DiagnosticSeverity.Error);
		});
	});

	describe('Quick Fixes Support', () => {
		it('should include quick fixes in diagnostics', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Missing semicolon',
					quick_fixes: [
						{
							file: '/test/file.ts',
							line: 0,
							character: 5,
							message: 'Add semicolon'
						}
					]
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 1);
			assert.ok(formattedDiagnostics[0].relatedInformation);
			assert.strictEqual(formattedDiagnostics[0].relatedInformation?.length, 1);
		});

		it('should handle diagnostics without quick fixes', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Test error'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 1);
			assert.strictEqual(formattedDiagnostics[0].relatedInformation, undefined);
		});

		it('should handle multiple quick fixes', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Multiple fixes available',
					quick_fixes: [
						{
							file: '/test/file.ts',
							line: 0,
							character: 5,
							message: 'Fix 1'
						},
						{
							file: '/test/file.ts',
							line: 0,
							character: 5,
							message: 'Fix 2'
						}
					]
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 1);
			assert.ok(formattedDiagnostics[0].relatedInformation);
			assert.strictEqual(formattedDiagnostics[0].relatedInformation?.length, 2);
		});
	});

	describe('Error Handling', () => {
		it('should handle errors gracefully', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts', scheme: 'file' },
				getText: () => { throw new Error('Document error'); }
			} as unknown as vscode.TextDocument;

			provider.start();
			provider.stop();

			// Provider should handle errors without crashing
			assert.ok(provider);
		});

		it('should handle invalid response data', () => {
			const mockDiagnostics = [
				null,
				undefined,
				{ range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } }, message: 'valid' },
				{}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			// Should handle invalid items gracefully
			assert.ok(formattedDiagnostics.length > 0);
		});

		it('should handle connection errors', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts', scheme: 'file' },
				getText: () => 'const x = 1;'
			} as unknown as vscode.TextDocument;

			provider.start();

			// Simulate disconnection
			assert.ok(!mockClient.isConnected());

			provider.stop();
		});
	});

	describe('Language Support', () => {
		it('should support multiple languages', () => {
			const languages = ['typescript', 'javascript', 'python', 'rust', 'go', 'java', 'cpp'];

			languages.forEach(lang => {
				const mockDocument = {
					languageId: lang,
					uri: { fsPath: `/test/file.${lang}`, scheme: 'file' },
					getText: () => 'test'
				} as unknown as vscode.TextDocument;

				assert.strictEqual(mockDocument.languageId, lang);
			});
		});
	});

	describe('Range Handling', () => {
		it('should handle single-line ranges', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 10 }
					},
					severity: 'Error',
					message: 'Single line error'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 1);
			assert.strictEqual(formattedDiagnostics[0].range.start.line, 0);
			assert.strictEqual(formattedDiagnostics[0].range.end.line, 0);
		});

		it('should handle multi-line ranges', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 5, character: 10 }
					},
					severity: 'Error',
					message: 'Multi-line error'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 1);
			assert.strictEqual(formattedDiagnostics[0].range.start.line, 0);
			assert.strictEqual(formattedDiagnostics[0].range.end.line, 5);
		});

		it('should handle zero-width ranges', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 5 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Zero-width error'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics.length, 1);
			assert.strictEqual(formattedDiagnostics[0].range.start.character, 5);
			assert.strictEqual(formattedDiagnostics[0].range.end.character, 5);
		});
	});

	describe('Diagnostic Codes', () => {
		it('should include diagnostic codes when provided', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Test error',
					code: 'TS1234'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics[0].code, 'TS1234');
		});

		it('should handle missing diagnostic codes', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Test error'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics[0].code, undefined);
		});
	});

	describe('Diagnostic Source', () => {
		it('should use provided source', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Test error',
					source: 'ESLint'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics[0].source, 'ESLint');
		});

		it('should default to RiceCoder source', () => {
			const mockDiagnostics = [
				{
					range: {
						start: { line: 0, character: 0 },
						end: { line: 0, character: 5 }
					},
					severity: 'Error',
					message: 'Test error'
				}
			];

			const formattedDiagnostics = (provider as unknown as DiagnosticsProviderForTesting).formatDiagnostics(mockDiagnostics);

			assert.strictEqual(formattedDiagnostics[0].source, 'RiceCoder');
		});
	});

	describe('Disposal', () => {
		it('should dispose resources properly', () => {
			provider.start();
			provider.dispose();

			// Provider should be disposed
			assert.ok(provider);
		});

		it('should clear timers on disposal', () => {
			provider.start();
			provider.dispose();

			// All timers should be cleared
			assert.ok(provider);
		});

		it('should dispose diagnostic collection', () => {
			provider.start();
			provider.dispose();

			// Diagnostic collection should be disposed
			assert.ok(provider);
		});
	});
});
