import * as vscode from 'vscode';
import { RicecoderClient } from './client/ricecoderClient';
import { CompletionProvider } from './providers/completionProvider';
import { DiagnosticsProvider } from './providers/diagnosticsProvider';
import { HoverProvider } from './providers/hoverProvider';
import { CommandHandler } from './commands/commandHandler';
import { showCommandPaletteHelp, showKeyboardShortcutsHelp } from './commands/commandPaletteIntegration';
import { SettingsManager, RicecoderSettings } from './settings/settingsManager';

let client: RicecoderClient;
let completionProvider: CompletionProvider;
let diagnosticsProvider: DiagnosticsProvider;
let hoverProvider: HoverProvider;
let commandHandler: CommandHandler;
let settingsManager: SettingsManager;

/**
 * Activate the RiceCoder extension
 * 
 * This function is called when the extension is activated. It:
 * 1. Initializes the settings manager for VS Code configuration
 * 2. Validates settings and shows errors if invalid
 * 3. Initializes the RiceCoder client for communication with the backend
 * 4. Registers IDE providers (completion, diagnostics, hover)
 * 5. Registers commands for the command palette
 * 6. Sets up configuration watchers for hot-reload
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
	try {
		// Initialize settings manager
		settingsManager = new SettingsManager();
		const settings = settingsManager.getSettings();

		// Validate settings
		const validationResult = settingsManager.validateSettings(settings);
		if (!validationResult.valid) {
			console.error('RiceCoder settings validation failed:', validationResult.errors);
			vscode.window.showErrorMessage(
				'RiceCoder settings validation failed. Please check your settings.',
				'Open Settings'
			).then((selection) => {
				if (selection === 'Open Settings') {
					vscode.commands.executeCommand('workbench.action.openSettings', 'ricecoder');
				}
			});
			return;
		}

		if (!settings.enabled) {
			console.log('RiceCoder extension is disabled');
			return;
		}

		// Initialize RiceCoder client with validated settings
		client = new RicecoderClient(settings.serverHost, settings.serverPort, settings.requestTimeout);
		await client.connect();

		// Initialize providers
		completionProvider = new CompletionProvider(client);
		diagnosticsProvider = new DiagnosticsProvider(client);
		hoverProvider = new HoverProvider(client);
		commandHandler = new CommandHandler(client);

		// Register completion provider
		if (settings.completionEnabled) {
			const completionDisposable = vscode.languages.registerCompletionItemProvider(
				{ scheme: 'file' },
				completionProvider,
				'.'
			);
			context.subscriptions.push(completionDisposable);
		}

		// Register diagnostics provider
		if (settings.diagnosticsEnabled) {
			diagnosticsProvider.start();
			context.subscriptions.push(diagnosticsProvider);
		}

		// Register hover provider
		if (settings.hoverEnabled) {
			const hoverDisposable = vscode.languages.registerHoverProvider(
				{ scheme: 'file' },
				hoverProvider
			);
			context.subscriptions.push(hoverDisposable);
		}

		// Register commands
		commandHandler.registerCommands(context);

		// Register help commands for command palette
		const helpCommand = vscode.commands.registerCommand('ricecoder.help', async () => {
			await showCommandPaletteHelp();
		});
		context.subscriptions.push(helpCommand);

		const shortcutsCommand = vscode.commands.registerCommand('ricecoder.shortcuts', async () => {
			await showKeyboardShortcutsHelp();
		});
		context.subscriptions.push(shortcutsCommand);

		// Register settings change handler
		const settingsChangeDisposable = settingsManager.onSettingsChanged(
			async (newSettings) => {
				await handleSettingsChange(newSettings);
			}
		);
		context.subscriptions.push(settingsChangeDisposable);

		console.log('RiceCoder extension activated successfully');
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		vscode.window.showErrorMessage(`Failed to activate RiceCoder: ${message}`);
		console.error('Failed to activate RiceCoder:', error);
	}
}

/**
 * Handle settings changes
 * 
 * When settings change, we need to:
 * 1. Validate the new settings
 * 2. Reconnect to the server if host/port changed
 * 3. Update provider settings
 * 4. Notify user of changes
 */
async function handleSettingsChange(newSettings: RicecoderSettings): Promise<void> {
	try {
		// Validate new settings
		const validationResult = settingsManager.validateSettings(newSettings);
		if (!validationResult.valid) {
			console.error('Settings validation failed:', validationResult.errors);
			vscode.window.showErrorMessage(
				'RiceCoder settings validation failed. Please check your settings.',
				'Open Settings'
			).then((selection) => {
				if (selection === 'Open Settings') {
					vscode.commands.executeCommand('workbench.action.openSettings', 'ricecoder');
				}
			});
			return;
		}

		// Handle enabled/disabled state
		if (!newSettings.enabled) {
			if (client) {
				await client.disconnect();
			}
			if (diagnosticsProvider) {
				diagnosticsProvider.stop();
			}
			vscode.window.showInformationMessage('RiceCoder extension disabled');
			return;
		}

		// Reconnect if server settings changed
		if (client && (client.getHost() !== newSettings.serverHost || client.getPort() !== newSettings.serverPort)) {
			await client.disconnect();
			client = new RicecoderClient(newSettings.serverHost, newSettings.serverPort, newSettings.requestTimeout);
			await client.connect();
			vscode.window.showInformationMessage('RiceCoder reconnected to server');
		}

		// Update request timeout
		if (client) {
			client.setRequestTimeout(newSettings.requestTimeout);
		}

		// Log settings change
		if (newSettings.debugMode) {
			console.log('RiceCoder settings updated:', newSettings);
		}
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		vscode.window.showErrorMessage(`Failed to update RiceCoder settings: ${message}`);
		console.error('Failed to update RiceCoder settings:', error);
	}
}

/**
 * Deactivate the RiceCoder extension
 * 
 * This function is called when the extension is deactivated. It:
 * 1. Disconnects from the RiceCoder server
 * 2. Cleans up resources
 * 3. Disposes settings manager
 */
export async function deactivate(): Promise<void> {
	try {
		if (client) {
			await client.disconnect();
		}
		if (diagnosticsProvider) {
			diagnosticsProvider.stop();
		}
		if (settingsManager) {
			settingsManager.dispose();
		}
		console.log('RiceCoder extension deactivated');
	} catch (error) {
		console.error('Error during deactivation:', error);
	}
}
