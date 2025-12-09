import * as vscode from 'vscode';
import { RicecoderClient } from './client/ricecoderClient';
import { CompletionProvider } from './providers/completionProvider';
import { DiagnosticsProvider } from './providers/diagnosticsProvider';
import { HoverProvider } from './providers/hoverProvider';
import { CommandHandler } from './commands/commandHandler';

let client: RicecoderClient;
let completionProvider: CompletionProvider;
let diagnosticsProvider: DiagnosticsProvider;
let hoverProvider: HoverProvider;
let commandHandler: CommandHandler;

/**
 * Activate the RiceCoder extension
 * 
 * This function is called when the extension is activated. It:
 * 1. Initializes the RiceCoder client for communication with the backend
 * 2. Registers IDE providers (completion, diagnostics, hover)
 * 3. Registers commands for the command palette
 * 4. Sets up configuration watchers for hot-reload
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
	try {
		// Get configuration
		const config = vscode.workspace.getConfiguration('ricecoder');
		const enabled = config.get<boolean>('enabled', true);

		if (!enabled) {
			console.log('RiceCoder extension is disabled');
			return;
		}

		// Initialize RiceCoder client
		const serverHost = config.get<string>('serverHost', 'localhost');
		const serverPort = config.get<number>('serverPort', 9000);
		const requestTimeout = config.get<number>('requestTimeout', 5000);

		client = new RicecoderClient(serverHost, serverPort, requestTimeout);
		await client.connect();

		// Initialize providers
		completionProvider = new CompletionProvider(client);
		diagnosticsProvider = new DiagnosticsProvider(client);
		hoverProvider = new HoverProvider(client);
		commandHandler = new CommandHandler(client);

		// Register completion provider
		if (config.get<boolean>('completionEnabled', true)) {
			const completionDisposable = vscode.languages.registerCompletionItemProvider(
				{ scheme: 'file' },
				completionProvider,
				'.'
			);
			context.subscriptions.push(completionDisposable);
		}

		// Register diagnostics provider
		if (config.get<boolean>('diagnosticsEnabled', true)) {
			diagnosticsProvider.start();
			context.subscriptions.push(diagnosticsProvider);
		}

		// Register hover provider
		if (config.get<boolean>('hoverEnabled', true)) {
			const hoverDisposable = vscode.languages.registerHoverProvider(
				{ scheme: 'file' },
				hoverProvider
			);
			context.subscriptions.push(hoverDisposable);
		}

		// Register commands
		commandHandler.registerCommands(context);

		// Watch for configuration changes
		const configWatcher = vscode.workspace.onDidChangeConfiguration(
			async (event) => {
				if (event.affectsConfiguration('ricecoder')) {
					await handleConfigurationChange(context);
				}
			}
		);
		context.subscriptions.push(configWatcher);

		console.log('RiceCoder extension activated successfully');
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		vscode.window.showErrorMessage(`Failed to activate RiceCoder: ${message}`);
		console.error('Failed to activate RiceCoder:', error);
	}
}

/**
 * Handle configuration changes
 * 
 * When configuration changes, we need to:
 * 1. Reconnect to the server if host/port changed
 * 2. Update provider settings
 * 3. Notify user of changes
 */
async function handleConfigurationChange(context: vscode.ExtensionContext): Promise<void> {
	try {
		const config = vscode.workspace.getConfiguration('ricecoder');
		const serverHost = config.get<string>('serverHost', 'localhost');
		const serverPort = config.get<number>('serverPort', 9000);
		const requestTimeout = config.get<number>('requestTimeout', 5000);

		// Reconnect if server settings changed
		if (client && (client.getHost() !== serverHost || client.getPort() !== serverPort)) {
			await client.disconnect();
			client = new RicecoderClient(serverHost, serverPort, requestTimeout);
			await client.connect();
			vscode.window.showInformationMessage('RiceCoder reconnected to server');
		}

		// Update request timeout
		if (client) {
			client.setRequestTimeout(requestTimeout);
		}
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		vscode.window.showErrorMessage(`Failed to update RiceCoder configuration: ${message}`);
	}
}

/**
 * Deactivate the RiceCoder extension
 * 
 * This function is called when the extension is deactivated. It:
 * 1. Disconnects from the RiceCoder server
 * 2. Cleans up resources
 */
export async function deactivate(): Promise<void> {
	try {
		if (client) {
			await client.disconnect();
		}
		if (diagnosticsProvider) {
			diagnosticsProvider.stop();
		}
		console.log('RiceCoder extension deactivated');
	} catch (error) {
		console.error('Error during deactivation:', error);
	}
}
