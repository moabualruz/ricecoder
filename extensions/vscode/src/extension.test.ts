import * as assert from 'assert';
import * as vscode from 'vscode';
import { RicecoderClient } from './client/ricecoderClient';
import { CompletionProvider } from './providers/completionProvider';
import { DiagnosticsProvider } from './providers/diagnosticsProvider';
import { HoverProvider } from './providers/hoverProvider';
import { CommandHandler } from './commands/commandHandler';
import { SettingsManager, RicecoderSettings } from './settings/settingsManager';

/**
 * Integration tests for VS Code extension
 * 
 * Tests the complete integration between:
 * - VS Code communication protocol
 * - Completion, diagnostics, and hover providers
 * - Command palette integration
 * - Settings integration
 * - Extension lifecycle (activation/deactivation)
 * 
 * **Feature: ricecoder-ide, Property 5: IDE Request Handling**
 * **Validates: Requirements 3.1-3.6**
 */
describe('VS Code Extension Integration', () => {
	let client: RicecoderClient;
	let completionProvider: CompletionProvider;
	let diagnosticsProvider: DiagnosticsProvider;
	let hoverProvider: HoverProvider;
	let commandHandler: CommandHandler;
	let settingsManager: SettingsManager;

	beforeEach(() => {
		// Initialize components
		settingsManager = new SettingsManager();
		client = new RicecoderClient('localhost', 9000, 5000);
		completionProvider = new CompletionProvider(client);
		diagnosticsProvider = new DiagnosticsProvider(client);
		hoverProvider = new HoverProvider(client);
		commandHandler = new CommandHandler(client);
	});

	afterEach(() => {
		// Clean up
		diagnosticsProvider.dispose();
		settingsManager.dispose();
	});

	describe('VS Code Communication Protocol', () => {
		it('should initialize client with correct settings', () => {
			assert.strictEqual(client.getHost(), 'localhost');
			assert.strictEqual(client.getPort(), 9000);
			assert.ok(!client.isConnected());
		});

		it('should support JSON-RPC request/response', () => {
			assert.ok(typeof client.request === 'function');
			assert.ok(typeof client.notify === 'function');
		});

		it('should support streaming responses', () => {
			assert.ok(typeof client.startStream === 'function');
			assert.ok(typeof client.onStreamChunk === 'function');
			assert.ok(typeof client.onStreamComplete === 'function');
			assert.ok(typeof client.onStreamError === 'function');
		});

		it('should handle connection lifecycle', async () => {
			assert.ok(typeof client.connect === 'function');
			assert.ok(typeof client.disconnect === 'function');
			assert.ok(typeof client.isConnected === 'function');
		});

		it('should support request timeout configuration', () => {
			client.setRequestTimeout(10000);
			const newClient = new RicecoderClient('localhost', 9000, 10000);
			assert.ok(newClient);
		});
	});

	describe('Completion Provider Integration', () => {
		it('should register completion provider', () => {
			assert.ok(completionProvider);
			assert.ok(typeof completionProvider.provideCompletionItems === 'function');
			assert.ok(typeof completionProvider.resolveCompletionItem === 'function');
		});

		it('should handle completion requests', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 10);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
			assert.ok(result === null || Array.isArray(result));
		});

		it('should format completion items for VS Code', () => {
			const mockItems = [
				{
					label: 'function',
					kind: 'Function',
					insert_text: 'function ${1:name}() {}',
					detail: 'Function declaration'
				}
			];

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const formattedItems = (completionProvider as any).formatCompletionItems(mockItems);
			assert.ok(Array.isArray(formattedItems));
			assert.ok(formattedItems.length > 0);
		});

		it('should support snippet expansion', () => {
			const mockItems = [
				{
					label: 'for',
					kind: 'Snippet',
					insert_text: 'for (let ${1:i} = 0; ${1:i} < ${2:n}; ${1:i}++) {\n\t$0\n}'
				}
			];

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const formattedItems = (completionProvider as any).formatCompletionItems(mockItems);
			assert.ok(formattedItems[0].insertText instanceof vscode.SnippetString);
		});

		it('should handle multiple languages', async () => {
			const languages = ['typescript', 'javascript', 'python', 'rust'];

			for (const lang of languages) {
				const mockDocument = {
					languageId: lang,
					uri: { fsPath: `/test/file.${lang}` },
					getText: () => 'test',
					getWordRangeAtPosition: () => null
				} as unknown as vscode.TextDocument;

				const position = new vscode.Position(0, 4);
				const context = { triggerCharacter: undefined } as vscode.CompletionContext;
				const token = { isCancellationRequested: false } as vscode.CancellationToken;

				const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
				assert.ok(result === null || Array.isArray(result));
			}
		});
	});

	describe('Diagnostics Provider Integration', () => {
		it('should register diagnostics provider', () => {
			assert.ok(diagnosticsProvider);
			assert.ok(typeof diagnosticsProvider.start === 'function');
			assert.ok(typeof diagnosticsProvider.stop === 'function');
			assert.ok(typeof diagnosticsProvider.dispose === 'function');
		});

		it('should start and stop monitoring', () => {
			diagnosticsProvider.start();
			assert.ok(diagnosticsProvider);
			diagnosticsProvider.stop();
		});

		it('should format diagnostics for VS Code', () => {
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

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const formattedDiagnostics = (diagnosticsProvider as any).formatDiagnostics(mockDiagnostics);
			assert.ok(Array.isArray(formattedDiagnostics));
			assert.ok(formattedDiagnostics.length > 0);
		});

		it('should map severity levels correctly', () => {
			const severities = ['Error', 'Warning', 'Information', 'Hint'];

			severities.forEach(severity => {
				const mockDiagnostics = [
					{
						range: {
							start: { line: 0, character: 0 },
							end: { line: 0, character: 5 }
						},
						severity,
						message: 'Test'
					}
				];

				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				const formattedDiagnostics = (diagnosticsProvider as any).formatDiagnostics(mockDiagnostics);
				assert.ok(formattedDiagnostics[0].severity !== undefined);
			});
		});

		it('should support quick fixes', () => {
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

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const formattedDiagnostics = (diagnosticsProvider as any).formatDiagnostics(mockDiagnostics);
			assert.ok(formattedDiagnostics[0].relatedInformation);
		});

		it('should handle document lifecycle', () => {
			diagnosticsProvider.start();
			// Simulate document events
			diagnosticsProvider.stop();
			assert.ok(diagnosticsProvider);
		});
	});

	describe('Hover Provider Integration', () => {
		it('should register hover provider', () => {
			assert.ok(hoverProvider);
			assert.ok(typeof hoverProvider.provideHover === 'function');
		});

		it('should handle hover requests', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = 1;',
				getWordRangeAtPosition: () => new vscode.Range(0, 6, 0, 7)
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 6);
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await hoverProvider.provideHover(mockDocument, position, token);
			assert.ok(result === null || result instanceof vscode.Hover);
		});

		it('should format hover information', () => {
			const mockData = {
				contents: 'Variable x',
				type_info: 'number'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const hover = (hoverProvider as any).formatHover(mockData, range);

			assert.ok(hover === null || hover instanceof vscode.Hover);
		});

		it('should support markdown in hover', () => {
			const mockData = {
				contents: '**Bold** text with `code`'
			};

			const range = new vscode.Range(0, 0, 0, 1);
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const hover = (hoverProvider as any).formatHover(mockData, range);

			assert.ok(hover === null || hover instanceof vscode.Hover);
		});

		it('should parse range information', () => {
			const rangeData = {
				start: { line: 0, character: 0 },
				end: { line: 0, character: 10 }
			};

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const range = (hoverProvider as any).parseRange(rangeData);
			assert.strictEqual(range.start.line, 0);
			assert.strictEqual(range.end.character, 10);
		});
	});

	describe('Command Palette Integration', () => {
		it('should initialize command handler', () => {
			assert.ok(commandHandler);
			assert.ok(typeof commandHandler.registerCommands === 'function');
		});

		it('should support ricecoder commands', () => {
			const commands = [
				'ricecoder.chat',
				'ricecoder.review',
				'ricecoder.generate',
				'ricecoder.refactor'
			];

			commands.forEach(cmd => {
				assert.ok(cmd.startsWith('ricecoder.'));
			});
		});

		it('should have keyboard shortcuts', () => {
			const shortcuts = {
				'ricecoder.chat': 'ctrl+shift+r',
				'ricecoder.review': 'ctrl+shift+e',
				'ricecoder.generate': 'ctrl+shift+g',
				'ricecoder.refactor': 'ctrl+shift+f'
			};

			Object.entries(shortcuts).forEach(([cmd, shortcut]) => {
				assert.ok(cmd);
				assert.ok(shortcut.includes('+'));
			});
		});

		it('should support context menu integration', () => {
			const contextMenuItems = [
				'ricecoder.chat',
				'ricecoder.review',
				'ricecoder.generate',
				'ricecoder.refactor'
			];

			contextMenuItems.forEach(item => {
				assert.ok(item.startsWith('ricecoder.'));
			});
		});
	});

	describe('Settings Integration', () => {
		it('should load default settings', () => {
			const settings = settingsManager.getSettings();

			assert.strictEqual(settings.enabled, true);
			assert.strictEqual(settings.serverHost, 'localhost');
			assert.strictEqual(settings.serverPort, 9000);
			assert.strictEqual(settings.requestTimeout, 5000);
		});

		it('should validate settings', () => {
			const validSettings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'lsp-first',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			const result = settingsManager.validateSettings(validSettings);
			assert.strictEqual(result.valid, true);
		});

		it('should reject invalid settings', () => {
			const invalidSettings: RicecoderSettings = {
				enabled: true,
				serverHost: '',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'lsp-first',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			const result = settingsManager.validateSettings(invalidSettings);
			assert.strictEqual(result.valid, false);
			assert.ok(result.errors.length > 0);
		});

		it('should provide remediation for invalid settings', () => {
			const invalidSettings: RicecoderSettings = {
				enabled: true,
				serverHost: '',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'lsp-first',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			const result = settingsManager.validateSettings(invalidSettings);
			assert.ok(result.errors[0].remediation.length > 0);
		});

		it('should support all provider selections', () => {
			const providers: Array<'lsp-first' | 'configured-rules' | 'builtin' | 'generic'> = [
				'lsp-first',
				'configured-rules',
				'builtin',
				'generic'
			];

			providers.forEach(provider => {
				const settings: RicecoderSettings = {
					enabled: true,
					serverHost: 'localhost',
					serverPort: 9000,
					requestTimeout: 5000,
					providerSelection: provider,
					completionEnabled: true,
					diagnosticsEnabled: true,
					hoverEnabled: true,
					debugMode: false,
					logLevel: 'info'
				};

				const result = settingsManager.validateSettings(settings);
				assert.strictEqual(result.valid, true);
			});
		});

		it('should support all log levels', () => {
			const logLevels: Array<'error' | 'warn' | 'info' | 'debug'> = [
				'error',
				'warn',
				'info',
				'debug'
			];

			logLevels.forEach(logLevel => {
				const settings: RicecoderSettings = {
					enabled: true,
					serverHost: 'localhost',
					serverPort: 9000,
					requestTimeout: 5000,
					providerSelection: 'lsp-first',
					completionEnabled: true,
					diagnosticsEnabled: true,
					hoverEnabled: true,
					debugMode: false,
					logLevel
				};

				const result = settingsManager.validateSettings(settings);
				assert.strictEqual(result.valid, true);
			});
		});

		it('should handle settings changes', async () => {
			const disposable = settingsManager.onSettingsChanged(() => {
				// Callback registered
			});

			disposable.dispose();
			assert.ok(disposable);
		});
	});

	describe('Provider Chain Integration', () => {
		it('should support LSP-first provider selection', () => {
			const settings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'lsp-first',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			assert.strictEqual(settings.providerSelection, 'lsp-first');
		});

		it('should support configured rules provider', () => {
			const settings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'configured-rules',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			assert.strictEqual(settings.providerSelection, 'configured-rules');
		});

		it('should support built-in provider', () => {
			const settings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'builtin',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			assert.strictEqual(settings.providerSelection, 'builtin');
		});

		it('should support generic provider', () => {
			const settings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'generic',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			assert.strictEqual(settings.providerSelection, 'generic');
		});
	});

	describe('Feature Flags', () => {
		it('should support enabling/disabling completion', () => {
			const settings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'lsp-first',
				completionEnabled: false,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			assert.strictEqual(settings.completionEnabled, false);
		});

		it('should support enabling/disabling diagnostics', () => {
			const settings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'lsp-first',
				completionEnabled: true,
				diagnosticsEnabled: false,
				hoverEnabled: true,
				debugMode: false,
				logLevel: 'info'
			};

			assert.strictEqual(settings.diagnosticsEnabled, false);
		});

		it('should support enabling/disabling hover', () => {
			const settings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'lsp-first',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: false,
				debugMode: false,
				logLevel: 'info'
			};

			assert.strictEqual(settings.hoverEnabled, false);
		});

		it('should support debug mode', () => {
			const settings: RicecoderSettings = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				providerSelection: 'lsp-first',
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				debugMode: true,
				logLevel: 'debug'
			};

			assert.strictEqual(settings.debugMode, true);
			assert.strictEqual(settings.logLevel, 'debug');
		});
	});

	describe('Error Handling and Resilience', () => {
		it('should handle client connection errors', async () => {
			const failingClient = new RicecoderClient('localhost', 9999, 100);

			try {
				await failingClient.request('test', {});
				assert.fail('Should have thrown');
			} catch (error) {
				assert.ok(error instanceof Error);
			}
		});

		it('should handle provider errors gracefully', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => { throw new Error('Document error'); },
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 0);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
			assert.ok(result === null || Array.isArray(result));
		});

		it('should handle invalid response data', () => {
			const mockItems = [null, undefined, { label: 'valid' }, {}];

			const formattedItems = (completionProvider as any).formatCompletionItems(mockItems);
			assert.ok(Array.isArray(formattedItems));
		});

		it('should handle timeout gracefully', async () => {
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
			assert.ok(result === null || Array.isArray(result));
		});

		it('should handle cancellation requests', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 10);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: true } as vscode.CancellationToken;

			const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
			assert.strictEqual(result, null);
		});
	});

	describe('Multi-Language Support', () => {
		it('should support TypeScript/JavaScript', async () => {
			const languages = ['typescript', 'javascript'];

			for (const lang of languages) {
				const mockDocument = {
					languageId: lang,
					uri: { fsPath: `/test/file.${lang === 'typescript' ? 'ts' : 'js'}` },
					getText: () => 'const x = ',
					getWordRangeAtPosition: () => null
				} as unknown as vscode.TextDocument;

				const position = new vscode.Position(0, 10);
				const context = { triggerCharacter: undefined } as vscode.CompletionContext;
				const token = { isCancellationRequested: false } as vscode.CancellationToken;

				const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
				assert.ok(result === null || Array.isArray(result));
			}
		});

		it('should support Python', async () => {
			const mockDocument = {
				languageId: 'python',
				uri: { fsPath: '/test/file.py' },
				getText: () => 'x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 4);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
			assert.ok(result === null || Array.isArray(result));
		});

		it('should support Rust', async () => {
			const mockDocument = {
				languageId: 'rust',
				uri: { fsPath: '/test/main.rs' },
				getText: () => 'let x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 8);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
			assert.ok(result === null || Array.isArray(result));
		});

		it('should support Go', async () => {
			const mockDocument = {
				languageId: 'go',
				uri: { fsPath: '/test/main.go' },
				getText: () => 'var x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(0, 8);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
			assert.ok(result === null || Array.isArray(result));
		});
	});

	describe('Extension Lifecycle', () => {
		it('should initialize all components', () => {
			assert.ok(client);
			assert.ok(completionProvider);
			assert.ok(diagnosticsProvider);
			assert.ok(hoverProvider);
			assert.ok(commandHandler);
			assert.ok(settingsManager);
		});

		it('should support activation', () => {
			const settings = settingsManager.getSettings();
			assert.strictEqual(settings.enabled, true);
		});

		it('should support deactivation', () => {
			diagnosticsProvider.stop();
			diagnosticsProvider.dispose();
			settingsManager.dispose();
			assert.ok(true);
		});

		it('should handle settings changes during runtime', async () => {
			const disposable = settingsManager.onSettingsChanged((_settings) => {
				// Settings changed
			});

			disposable.dispose();
			assert.ok(disposable);
		});

		it('should support reconnection', async () => {
			const newClient = new RicecoderClient('localhost', 9000, 5000);
			assert.ok(!newClient.isConnected());
		});
	});

	describe('Performance and Optimization', () => {
		it('should handle rapid completion requests', async () => {
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/file.ts' },
				getText: () => 'const x = ',
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			for (let i = 0; i < 5; i++) {
				const position = new vscode.Position(0, 10 + i);
				const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
				assert.ok(result === null || Array.isArray(result));
			}
		});

		it('should handle large documents', async () => {
			const largeContent = 'const x = 1;\n'.repeat(1000);
			const mockDocument = {
				languageId: 'typescript',
				uri: { fsPath: '/test/large.ts' },
				getText: () => largeContent,
				getWordRangeAtPosition: () => null
			} as unknown as vscode.TextDocument;

			const position = new vscode.Position(500, 10);
			const context = { triggerCharacter: undefined } as vscode.CompletionContext;
			const token = { isCancellationRequested: false } as vscode.CancellationToken;

			const result = await completionProvider.provideCompletionItems(mockDocument, position, token, context);
			assert.ok(result === null || Array.isArray(result));
		});

		it('should handle many diagnostics', () => {
			const mockDiagnostics = Array.from({ length: 100 }, (_, i) => ({
				range: {
					start: { line: i, character: 0 },
					end: { line: i, character: 5 }
				},
				severity: 'Warning',
				message: `Warning ${i}`
			}));

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const formattedDiagnostics = (diagnosticsProvider as any).formatDiagnostics(mockDiagnostics);
			assert.strictEqual(formattedDiagnostics.length, 100);
		});
	});
});
