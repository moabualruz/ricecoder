import * as vscode from 'vscode';

/**
 * VS Code Settings Configuration
 * 
 * Represents the complete settings structure for RiceCoder
 */
export interface RicecoderSettings {
	// Connection settings
	enabled: boolean;
	serverHost: string;
	serverPort: number;
	requestTimeout: number;

	// Provider settings
	providerSelection: 'lsp-first' | 'configured-rules' | 'builtin' | 'generic';

	// Feature settings
	completionEnabled: boolean;
	diagnosticsEnabled: boolean;
	hoverEnabled: boolean;

	// Advanced settings
	debugMode: boolean;
	logLevel: 'error' | 'warn' | 'info' | 'debug';
}

/**
 * Settings validation result
 */
export interface SettingsValidationResult {
	valid: boolean;
	errors: SettingsError[];
	warnings: SettingsWarning[];
}

/**
 * Settings error with remediation steps
 */
export interface SettingsError {
	field: string;
	message: string;
	value: unknown;
	remediation: string;
}

/**
 * Settings warning
 */
export interface SettingsWarning {
	field: string;
	message: string;
	suggestion: string;
}

/**
 * Settings Manager for VS Code
 * 
 * Handles:
 * - Reading VS Code settings
 * - Validating settings with clear error messages
 * - Providing remediation steps for invalid settings
 * - Watching for settings changes
 * - Applying settings to providers
 */
export class SettingsManager {
	private settings: RicecoderSettings;
	private onSettingsChangedCallbacks: Array<(settings: RicecoderSettings) => void> = [];
	private disposables: vscode.Disposable[] = [];

	constructor() {
		this.settings = this.loadSettings();
		this.setupWatchers();
	}

	/**
	 * Load settings from VS Code configuration
	 */
	private loadSettings(): RicecoderSettings {
		const config = vscode.workspace.getConfiguration('ricecoder');

		return {
			enabled: config.get<boolean>('enabled', true),
			serverHost: config.get<string>('serverHost', 'localhost'),
			serverPort: config.get<number>('serverPort', 9000),
			requestTimeout: config.get<number>('requestTimeout', 5000),
			providerSelection: config.get<'lsp-first' | 'configured-rules' | 'builtin' | 'generic'>('providerSelection', 'lsp-first'),
			completionEnabled: config.get<boolean>('completionEnabled', true),
			diagnosticsEnabled: config.get<boolean>('diagnosticsEnabled', true),
			hoverEnabled: config.get<boolean>('hoverEnabled', true),
			debugMode: config.get<boolean>('debugMode', false),
			logLevel: config.get<'error' | 'warn' | 'info' | 'debug'>('logLevel', 'info'),
		};
	}

	/**
	 * Setup watchers for configuration changes
	 */
	private setupWatchers(): void {
		const watcher = vscode.workspace.onDidChangeConfiguration((event) => {
			if (event.affectsConfiguration('ricecoder')) {
				const newSettings = this.loadSettings();
				const validationResult = this.validateSettings(newSettings);

				if (!validationResult.valid) {
					this.showValidationErrors(validationResult.errors);
					return;
				}

				if (validationResult.warnings.length > 0) {
					this.showValidationWarnings(validationResult.warnings);
				}

				this.settings = newSettings;
				this.notifySettingsChanged(newSettings);
			}
		});

		this.disposables.push(watcher);
	}

	/**
	 * Get current settings
	 */
	getSettings(): RicecoderSettings {
		return { ...this.settings };
	}

	/**
	 * Validate settings
	 */
	validateSettings(settings: RicecoderSettings): SettingsValidationResult {
		const errors: SettingsError[] = [];
		const warnings: SettingsWarning[] = [];

		// Validate serverHost
		if (!settings.serverHost || typeof settings.serverHost !== 'string') {
			errors.push({
				field: 'serverHost',
				message: 'Server host must be a non-empty string',
				value: settings.serverHost,
				remediation: 'Set ricecoder.serverHost to a valid hostname or IP address (e.g., "localhost" or "127.0.0.1")',
			});
		}

		// Validate serverPort
		if (!Number.isInteger(settings.serverPort) || settings.serverPort < 1 || settings.serverPort > 65535) {
			errors.push({
				field: 'serverPort',
				message: 'Server port must be an integer between 1 and 65535',
				value: settings.serverPort,
				remediation: 'Set ricecoder.serverPort to a valid port number (e.g., 9000)',
			});
		}

		// Validate requestTimeout
		if (!Number.isInteger(settings.requestTimeout) || settings.requestTimeout < 100) {
			errors.push({
				field: 'requestTimeout',
				message: 'Request timeout must be an integer >= 100 milliseconds',
				value: settings.requestTimeout,
				remediation: 'Set ricecoder.requestTimeout to a value >= 100 (e.g., 5000 for 5 seconds)',
			});
		}

		// Validate providerSelection
		const validProviders = ['lsp-first', 'configured-rules', 'builtin', 'generic'];
		if (!validProviders.includes(settings.providerSelection)) {
			errors.push({
				field: 'providerSelection',
				message: `Provider selection must be one of: ${validProviders.join(', ')}`,
				value: settings.providerSelection,
				remediation: `Set ricecoder.providerSelection to one of: ${validProviders.join(', ')}`,
			});
		}

		// Validate logLevel
		const validLogLevels = ['error', 'warn', 'info', 'debug'];
		if (!validLogLevels.includes(settings.logLevel)) {
			errors.push({
				field: 'logLevel',
				message: `Log level must be one of: ${validLogLevels.join(', ')}`,
				value: settings.logLevel,
				remediation: `Set ricecoder.logLevel to one of: ${validLogLevels.join(', ')}`,
			});
		}

		// Warnings
		if (settings.requestTimeout > 30000) {
			warnings.push({
				field: 'requestTimeout',
				message: 'Request timeout is very high (> 30 seconds)',
				suggestion: 'Consider reducing ricecoder.requestTimeout for better responsiveness',
			});
		}

		if (settings.debugMode && settings.logLevel !== 'debug') {
			warnings.push({
				field: 'logLevel',
				message: 'Debug mode is enabled but log level is not set to debug',
				suggestion: 'Set ricecoder.logLevel to "debug" for detailed logging',
			});
		}

		return {
			valid: errors.length === 0,
			errors,
			warnings,
		};
	}

	/**
	 * Show validation errors to user
	 */
	private showValidationErrors(errors: SettingsError[]): void {
		const errorMessages = errors.map((error) => {
			return `${error.field}: ${error.message}\n\nRemediation: ${error.remediation}`;
		}).join('\n\n---\n\n');

		vscode.window.showErrorMessage(
			`RiceCoder Settings Validation Failed:\n\n${errorMessages}`,
			'Open Settings'
		).then((selection) => {
			if (selection === 'Open Settings') {
				vscode.commands.executeCommand('workbench.action.openSettings', 'ricecoder');
			}
		});
	}

	/**
	 * Show validation warnings to user
	 */
	private showValidationWarnings(warnings: SettingsWarning[]): void {
		const warningMessages = warnings.map((warning) => {
			return `${warning.field}: ${warning.message}\n\nSuggestion: ${warning.suggestion}`;
		}).join('\n\n---\n\n');

		vscode.window.showWarningMessage(
			`RiceCoder Settings Warnings:\n\n${warningMessages}`,
			'Open Settings'
		).then((selection) => {
			if (selection === 'Open Settings') {
				vscode.commands.executeCommand('workbench.action.openSettings', 'ricecoder');
			}
		});
	}

	/**
	 * Register callback for settings changes
	 */
	onSettingsChanged(callback: (settings: RicecoderSettings) => void): vscode.Disposable {
		this.onSettingsChangedCallbacks.push(callback);

		return {
			dispose: () => {
				const index = this.onSettingsChangedCallbacks.indexOf(callback);
				if (index > -1) {
					this.onSettingsChangedCallbacks.splice(index, 1);
				}
			},
		};
	}

	/**
	 * Notify all listeners of settings change
	 */
	private notifySettingsChanged(settings: RicecoderSettings): void {
		for (const callback of this.onSettingsChangedCallbacks) {
			try {
				callback(settings);
			} catch (error) {
				console.error('Error in settings change callback:', error);
			}
		}
	}

	/**
	 * Update a specific setting
	 */
	async updateSetting(key: string, value: unknown): Promise<void> {
		const config = vscode.workspace.getConfiguration('ricecoder');
		await config.update(key, value, vscode.ConfigurationTarget.Global);
	}

	/**
	 * Get a specific setting
	 */
	getSetting<T>(key: string, defaultValue: T): T {
		const config = vscode.workspace.getConfiguration('ricecoder');
		return config.get<T>(key, defaultValue);
	}

	/**
	 * Dispose resources
	 */
	dispose(): void {
		for (const disposable of this.disposables) {
			disposable.dispose();
		}
		this.disposables = [];
		this.onSettingsChangedCallbacks = [];
	}
}
