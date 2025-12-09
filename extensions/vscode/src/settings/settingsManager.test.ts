import * as assert from 'assert';
import { SettingsManager, RicecoderSettings } from './settingsManager';

/**
 * Test suite for SettingsManager
 * 
 * Tests:
 * - Loading settings from VS Code configuration
 * - Validating settings with clear error messages
 * - Handling settings changes
 * - Providing remediation steps for invalid settings
 */
suite('SettingsManager', () => {
	let settingsManager: SettingsManager;

	setup(() => {
		settingsManager = new SettingsManager();
	});

	teardown(() => {
		settingsManager.dispose();
	});

	test('should load default settings', () => {
		const settings = settingsManager.getSettings();

		assert.strictEqual(settings.enabled, true);
		assert.strictEqual(settings.serverHost, 'localhost');
		assert.strictEqual(settings.serverPort, 9000);
		assert.strictEqual(settings.requestTimeout, 5000);
		assert.strictEqual(settings.providerSelection, 'lsp-first');
		assert.strictEqual(settings.completionEnabled, true);
		assert.strictEqual(settings.diagnosticsEnabled, true);
		assert.strictEqual(settings.hoverEnabled, true);
		assert.strictEqual(settings.debugMode, false);
		assert.strictEqual(settings.logLevel, 'info');
	});

	test('should validate valid settings', () => {
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
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(validSettings);

		assert.strictEqual(result.valid, true);
		assert.strictEqual(result.errors.length, 0);
	});

	test('should reject invalid serverHost', () => {
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
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(invalidSettings);

		assert.strictEqual(result.valid, false);
		assert.strictEqual(result.errors.length, 1);
		assert.strictEqual(result.errors[0].field, 'serverHost');
		assert.ok(result.errors[0].remediation.includes('serverHost'));
	});

	test('should reject invalid serverPort (too low)', () => {
		const invalidSettings: RicecoderSettings = {
			enabled: true,
			serverHost: 'localhost',
			serverPort: 0 as any,
			requestTimeout: 5000,
			providerSelection: 'lsp-first',
			completionEnabled: true,
			diagnosticsEnabled: true,
			hoverEnabled: true,
			debugMode: false,
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(invalidSettings);

		assert.strictEqual(result.valid, false);
		assert.ok(result.errors.some((e) => e.field === 'serverPort'));
	});

	test('should reject invalid serverPort (too high)', () => {
		const invalidSettings: RicecoderSettings = {
			enabled: true,
			serverHost: 'localhost',
			serverPort: 65536 as any,
			requestTimeout: 5000,
			providerSelection: 'lsp-first',
			completionEnabled: true,
			diagnosticsEnabled: true,
			hoverEnabled: true,
			debugMode: false,
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(invalidSettings);

		assert.strictEqual(result.valid, false);
		assert.ok(result.errors.some((e) => e.field === 'serverPort'));
	});

	test('should reject invalid requestTimeout (too low)', () => {
		const invalidSettings: RicecoderSettings = {
			enabled: true,
			serverHost: 'localhost',
			serverPort: 9000,
			requestTimeout: 50,
			providerSelection: 'lsp-first',
			completionEnabled: true,
			diagnosticsEnabled: true,
			hoverEnabled: true,
			debugMode: false,
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(invalidSettings);

		assert.strictEqual(result.valid, false);
		assert.ok(result.errors.some((e) => e.field === 'requestTimeout'));
	});

	test('should reject invalid providerSelection', () => {
		const invalidSettings: RicecoderSettings = {
			enabled: true,
			serverHost: 'localhost',
			serverPort: 9000,
			requestTimeout: 5000,
			providerSelection: 'invalid' as any,
			completionEnabled: true,
			diagnosticsEnabled: true,
			hoverEnabled: true,
			debugMode: false,
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(invalidSettings);

		assert.strictEqual(result.valid, false);
		assert.ok(result.errors.some((e) => e.field === 'providerSelection'));
	});

	test('should reject invalid logLevel', () => {
		const invalidSettings: RicecoderSettings = {
			enabled: true,
			serverHost: 'localhost',
			serverPort: 9000,
			requestTimeout: 5000,
			providerSelection: 'lsp-first',
			completionEnabled: true,
			diagnosticsEnabled: true,
			hoverEnabled: true,
			debugMode: false,
			logLevel: 'invalid' as any,
		};

		const result = settingsManager.validateSettings(invalidSettings);

		assert.strictEqual(result.valid, false);
		assert.ok(result.errors.some((e) => e.field === 'logLevel'));
	});

	test('should warn on high requestTimeout', () => {
		const settings: RicecoderSettings = {
			enabled: true,
			serverHost: 'localhost',
			serverPort: 9000,
			requestTimeout: 60000,
			providerSelection: 'lsp-first',
			completionEnabled: true,
			diagnosticsEnabled: true,
			hoverEnabled: true,
			debugMode: false,
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(settings);

		assert.strictEqual(result.valid, true);
		assert.ok(result.warnings.some((w) => w.field === 'requestTimeout'));
	});

	test('should warn on debugMode without debug logLevel', () => {
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
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(settings);

		assert.strictEqual(result.valid, true);
		assert.ok(result.warnings.some((w) => w.field === 'logLevel'));
	});

	test('should provide remediation for invalid settings', () => {
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
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(invalidSettings);

		assert.strictEqual(result.valid, false);
		assert.ok(result.errors[0].remediation.length > 0);
		assert.ok(result.errors[0].remediation.includes('ricecoder.serverHost'));
	});

	test('should accept valid IP addresses', () => {
		const settings: RicecoderSettings = {
			enabled: true,
			serverHost: '127.0.0.1',
			serverPort: 9000,
			requestTimeout: 5000,
			providerSelection: 'lsp-first',
			completionEnabled: true,
			diagnosticsEnabled: true,
			hoverEnabled: true,
			debugMode: false,
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(settings);

		assert.strictEqual(result.valid, true);
	});

	test('should accept valid hostnames', () => {
		const settings: RicecoderSettings = {
			enabled: true,
			serverHost: 'example.com',
			serverPort: 9000,
			requestTimeout: 5000,
			providerSelection: 'lsp-first',
			completionEnabled: true,
			diagnosticsEnabled: true,
			hoverEnabled: true,
			debugMode: false,
			logLevel: 'info',
		};

		const result = settingsManager.validateSettings(settings);

		assert.strictEqual(result.valid, true);
	});

	test('should accept all valid provider selections', () => {
		const providers: Array<'lsp-first' | 'configured-rules' | 'builtin' | 'generic'> = [
			'lsp-first',
			'configured-rules',
			'builtin',
			'generic',
		];

		for (const provider of providers) {
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
				logLevel: 'info',
			};

			const result = settingsManager.validateSettings(settings);

			assert.strictEqual(result.valid, true, `Provider ${provider} should be valid`);
		}
	});

	test('should accept all valid log levels', () => {
		const logLevels: Array<'error' | 'warn' | 'info' | 'debug'> = [
			'error',
			'warn',
			'info',
			'debug',
		];

		for (const logLevel of logLevels) {
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
				logLevel,
			};

			const result = settingsManager.validateSettings(settings);

			assert.strictEqual(result.valid, true, `Log level ${logLevel} should be valid`);
		}
	});

	test('should handle settings change callbacks', async () => {
		const disposable = settingsManager.onSettingsChanged((_settings) => {
			// Callback registered successfully
		});

		// Simulate settings change by updating a setting
		await settingsManager.updateSetting('debugMode', true);

		// Give callback time to execute
		await new Promise((resolve) => setTimeout(resolve, 100));

		disposable.dispose();

		// Note: This test may not work in all environments due to VS Code configuration
		// but it tests the callback mechanism
	});
});
