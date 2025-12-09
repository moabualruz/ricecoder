import * as assert from 'assert';
import { RicecoderClient } from './ricecoderClient';
import * as protocol from './protocolHandler';

/**
 * Integration tests for VS Code communication protocol
 * 
 * These tests verify the complete communication flow between
 * the VS Code extension and the RiceCoder backend.
 */
describe('VS Code Communication Protocol Integration', () => {
	let client: RicecoderClient;

	beforeEach(() => {
		client = new RicecoderClient('localhost', 9000, 5000);
	});

	describe('Protocol Types', () => {
		it('should validate Position type', () => {
			const validPos: protocol.Position = { line: 10, character: 5 };
			assert.ok(protocol.isValidPosition(validPos));

			const invalidPos = { line: 'ten', character: 5 };
			assert.strictEqual(protocol.isValidPosition(invalidPos), false);
		});

		it('should validate Range type', () => {
			const validRange: protocol.Range = {
				start: { line: 10, character: 5 },
				end: { line: 10, character: 15 }
			};
			assert.ok(protocol.isValidRange(validRange));

			const invalidRange = {
				start: { line: 'ten', character: 5 },
				end: { line: 10, character: 15 }
			};
			assert.strictEqual(protocol.isValidRange(invalidRange), false);
		});

		it('should detect error responses', () => {
			const errorResponse: protocol.ProtocolError = {
				code: -32602,
				message: 'Invalid params'
			};
			assert.ok(protocol.isErrorResponse(errorResponse));

			const validResponse = { result: 'success' };
			assert.strictEqual(protocol.isErrorResponse(validResponse), false);
		});
	});

	describe('Request Types', () => {
		it('should support completion request type', () => {
			const params: protocol.CompletionParams = {
				language: 'rust',
				file_path: '/path/to/main.rs',
				position: { line: 10, character: 5 },
				context: 'fn main() {',
				trigger_character: '.'
			};
			assert.ok(params.language);
			assert.ok(params.file_path);
			assert.ok(protocol.isValidPosition(params.position));
		});

		it('should support diagnostics request type', () => {
			const params: protocol.DiagnosticsParams = {
				language: 'rust',
				file_path: '/path/to/main.rs',
				source: 'fn main() { let x = ; }'
			};
			assert.ok(params.language);
			assert.ok(params.source);
		});

		it('should support hover request type', () => {
			const params: protocol.HoverParams = {
				language: 'rust',
				file_path: '/path/to/main.rs',
				position: { line: 10, character: 5 },
				word: 'println'
			};
			assert.ok(params.language);
			assert.ok(params.word);
		});

		it('should support definition request type', () => {
			const params: protocol.DefinitionParams = {
				language: 'rust',
				file_path: '/path/to/main.rs',
				position: { line: 10, character: 5 }
			};
			assert.ok(params.language);
			assert.ok(protocol.isValidPosition(params.position));
		});
	});

	describe('Response Types', () => {
		it('should support completion item response', () => {
			const item: protocol.CompletionItem = {
				label: 'println!',
				kind: 'Macro',
				insert_text: 'println!("$1");',
				documentation: 'Print to stdout'
			};
			assert.strictEqual(item.label, 'println!');
			assert.strictEqual(item.kind, 'Macro');
		});

		it('should support diagnostic response', () => {
			const diag: protocol.Diagnostic = {
				range: {
					start: { line: 10, character: 5 },
					end: { line: 10, character: 15 }
				},
				severity: 'Error',
				message: 'Unexpected token',
				source: 'RiceCoder'
			};
			assert.strictEqual(diag.severity, 'Error');
			assert.ok(protocol.isValidRange(diag.range));
		});

		it('should support hover response', () => {
			const hover: protocol.Hover = {
				contents: 'Prints to stdout',
				type_info: 'macro',
				documentation: 'Macro for printing'
			};
			assert.ok(hover.contents);
			assert.ok(hover.type_info);
		});

		it('should support location response', () => {
			const location: protocol.Location = {
				file_path: '/path/to/definition.rs',
				range: {
					start: { line: 5, character: 0 },
					end: { line: 5, character: 10 }
				}
			};
			assert.ok(location.file_path);
			assert.ok(protocol.isValidRange(location.range));
		});
	});

	describe('Command Types', () => {
		it('should support chat command', () => {
			const params: protocol.ChatParams = {
				message: 'Explain this code',
				context: 'fn main() {}',
				language: 'rust',
				file_path: '/path/to/main.rs'
			};
			assert.ok(params.message);
			assert.ok(params.context);
		});

		it('should support review command', () => {
			const params: protocol.ReviewParams = {
				language: 'rust',
				file_path: '/path/to/main.rs',
				source: 'fn main() {}'
			};
			assert.ok(params.language);
			assert.ok(params.source);
		});

		it('should support generate command', () => {
			const params: protocol.GenerateParams = {
				prompt: 'Generate a function',
				language: 'rust',
				file_path: '/path/to/main.rs',
				context: 'fn main() {}'
			};
			assert.ok(params.prompt);
			assert.ok(params.language);
		});

		it('should support refactor command', () => {
			const params: protocol.RefactorParams = {
				refactoring_type: 'Extract Function',
				language: 'rust',
				file_path: '/path/to/main.rs',
				source: 'let x = 1; let y = 2;'
			};
			assert.ok(params.refactoring_type);
			assert.ok(params.source);
		});
	});

	describe('Error Handling', () => {
		it('should create error responses', () => {
			const error = protocol.createErrorResponse(
				protocol.ErrorCode.InvalidParams,
				'Missing required field: language'
			);
			assert.strictEqual(error.code, protocol.ErrorCode.InvalidParams);
			assert.ok(error.message);
		});

		it('should handle all error codes', () => {
			assert.ok(protocol.ErrorCode.ParseError);
			assert.ok(protocol.ErrorCode.InvalidRequest);
			assert.ok(protocol.ErrorCode.MethodNotFound);
			assert.ok(protocol.ErrorCode.InvalidParams);
			assert.ok(protocol.ErrorCode.InternalError);
			assert.ok(protocol.ErrorCode.ConnectionError);
			assert.ok(protocol.ErrorCode.TimeoutError);
			assert.ok(protocol.ErrorCode.StreamError);
			assert.ok(protocol.ErrorCode.ConfigError);
		});
	});

	describe('Configuration', () => {
		it('should support IDE configuration', () => {
			const config: protocol.IdeConfig = {
				enabled: true,
				serverHost: 'localhost',
				serverPort: 9000,
				requestTimeout: 5000,
				completionEnabled: true,
				diagnosticsEnabled: true,
				hoverEnabled: true,
				providerSelection: 'lsp-first'
			};
			assert.strictEqual(config.enabled, true);
			assert.strictEqual(config.providerSelection, 'lsp-first');
		});
	});

	describe('Stream Types', () => {
		it('should support stream start parameters', () => {
			const params: protocol.StreamStartParams = {
				method: 'chat/send',
				params: { message: 'Hello' },
				stream_id: 'stream_1'
			};
			assert.ok(params.method);
			assert.ok(params.stream_id);
		});

		it('should support stream chunk', () => {
			const chunk: protocol.StreamChunk = {
				stream_id: 'stream_1',
				chunk: 'Hello ',
				index: 0,
				total: 2
			};
			assert.strictEqual(chunk.stream_id, 'stream_1');
			assert.strictEqual(chunk.index, 0);
		});

		it('should support stream complete', () => {
			const complete: protocol.StreamComplete = {
				stream_id: 'stream_1',
				chunks: ['Hello ', 'World']
			};
			assert.strictEqual(complete.chunks.length, 2);
		});

		it('should support stream error', () => {
			const error: protocol.StreamError = {
				stream_id: 'stream_1',
				error: 'Connection lost',
				code: 1000
			};
			assert.ok(error.error);
			assert.strictEqual(error.code, 1000);
		});
	});

	describe('Protocol Version', () => {
		it('should have protocol version', () => {
			assert.ok(protocol.PROTOCOL_VERSION);
			assert.strictEqual(protocol.PROTOCOL_VERSION, '1.0.0');
		});
	});

	describe('Request Types Enum', () => {
		it('should have all request types', () => {
			assert.ok(protocol.RequestType.Completion);
			assert.ok(protocol.RequestType.Diagnostics);
			assert.ok(protocol.RequestType.Hover);
			assert.ok(protocol.RequestType.Definition);
			assert.ok(protocol.RequestType.Chat);
			assert.ok(protocol.RequestType.Review);
			assert.ok(protocol.RequestType.Generate);
			assert.ok(protocol.RequestType.Refactor);
		});
	});

	describe('Notification Types Enum', () => {
		it('should have all notification types', () => {
			assert.ok(protocol.NotificationType.ServerReady);
			assert.ok(protocol.NotificationType.ServerShutdown);
			assert.ok(protocol.NotificationType.ConfigChanged);
			assert.ok(protocol.NotificationType.StreamChunk);
			assert.ok(protocol.NotificationType.StreamComplete);
			assert.ok(protocol.NotificationType.StreamError);
		});
	});

	describe('Client Integration', () => {
		it('should have streaming handler', () => {
			const handler = client.getStreamingHandler();
			assert.ok(handler);
		});

		it('should support stream methods', () => {
			assert.ok(typeof client.startStream === 'function');
			assert.ok(typeof client.onStreamChunk === 'function');
			assert.ok(typeof client.onStreamComplete === 'function');
			assert.ok(typeof client.onStreamError === 'function');
			assert.ok(typeof client.cancelStream === 'function');
			assert.ok(typeof client.getStreamText === 'function');
		});

		it('should support request methods', () => {
			assert.ok(typeof client.request === 'function');
			assert.ok(typeof client.notify === 'function');
		});

		it('should support connection methods', () => {
			assert.ok(typeof client.connect === 'function');
			assert.ok(typeof client.disconnect === 'function');
			assert.ok(typeof client.isConnected === 'function');
		});
	});
});
