import * as assert from 'assert';
import {
	validateType,
	validateNumberRange,
	validateNonEmptyString,
	validateEnum,
	validatePort,
	validateTimeout,
	validateHostname,
	getRemediationMessage,
	getErrorMessage,
	findValidationRule,
	VALIDATION_RULES,
} from './settingsValidator';

/**
 * Test suite for Settings Validator
 * 
 * Tests:
 * - Type validation
 * - Range validation
 * - Enum validation
 * - Port validation
 * - Timeout validation
 * - Hostname validation
 * - Error and remediation messages
 */
suite('SettingsValidator', () => {
	test('validateType should validate strings', () => {
		assert.strictEqual(validateType('hello', 'string'), true);
		assert.strictEqual(validateType(123, 'string'), false);
		assert.strictEqual(validateType(null, 'string'), false);
	});

	test('validateType should validate numbers', () => {
		assert.strictEqual(validateType(123, 'number'), true);
		assert.strictEqual(validateType(123.45, 'number'), true);
		assert.strictEqual(validateType('123', 'number'), false);
		assert.strictEqual(validateType(NaN, 'number'), false);
	});

	test('validateType should validate booleans', () => {
		assert.strictEqual(validateType(true, 'boolean'), true);
		assert.strictEqual(validateType(false, 'boolean'), true);
		assert.strictEqual(validateType(1, 'boolean'), false);
	});

	test('validateType should validate integers', () => {
		assert.strictEqual(validateType(123, 'integer'), true);
		assert.strictEqual(validateType(123.45, 'integer'), false);
		assert.strictEqual(validateType('123', 'integer'), false);
	});

	test('validateType should validate arrays', () => {
		assert.strictEqual(validateType([], 'array'), true);
		assert.strictEqual(validateType([1, 2, 3], 'array'), true);
		assert.strictEqual(validateType('array', 'array'), false);
	});

	test('validateType should validate objects', () => {
		assert.strictEqual(validateType({}, 'object'), true);
		assert.strictEqual(validateType({ a: 1 }, 'object'), true);
		assert.strictEqual(validateType([], 'object'), false);
		assert.strictEqual(validateType(null, 'object'), false);
	});

	test('validateNumberRange should validate numbers in range', () => {
		assert.strictEqual(validateNumberRange(5, 1, 10), true);
		assert.strictEqual(validateNumberRange(1, 1, 10), true);
		assert.strictEqual(validateNumberRange(10, 1, 10), true);
		assert.strictEqual(validateNumberRange(0, 1, 10), false);
		assert.strictEqual(validateNumberRange(11, 1, 10), false);
	});

	test('validateNonEmptyString should validate non-empty strings', () => {
		assert.strictEqual(validateNonEmptyString('hello'), true);
		assert.strictEqual(validateNonEmptyString(''), false);
		assert.strictEqual(validateNonEmptyString('   '), false);
		assert.strictEqual(validateNonEmptyString(123), false);
	});

	test('validateEnum should validate enum values', () => {
		const values = ['a', 'b', 'c'];
		assert.strictEqual(validateEnum('a', values), true);
		assert.strictEqual(validateEnum('b', values), true);
		assert.strictEqual(validateEnum('d', values), false);
		assert.strictEqual(validateEnum(123, values), false);
	});

	test('validatePort should validate port numbers', () => {
		assert.strictEqual(validatePort(80), true);
		assert.strictEqual(validatePort(443), true);
		assert.strictEqual(validatePort(9000), true);
		assert.strictEqual(validatePort(1), true);
		assert.strictEqual(validatePort(65535), true);
		assert.strictEqual(validatePort(0), false);
		assert.strictEqual(validatePort(65536), false);
		assert.strictEqual(validatePort(80.5), false);
		assert.strictEqual(validatePort('80'), false);
	});

	test('validateTimeout should validate timeout values', () => {
		assert.strictEqual(validateTimeout(100), true);
		assert.strictEqual(validateTimeout(5000), true);
		assert.strictEqual(validateTimeout(300000), true);
		assert.strictEqual(validateTimeout(99), false);
		assert.strictEqual(validateTimeout(300001), false);
		assert.strictEqual(validateTimeout(5000.5), false);
	});

	test('validateHostname should validate hostnames', () => {
		assert.strictEqual(validateHostname('localhost'), true);
		assert.strictEqual(validateHostname('example.com'), true);
		assert.strictEqual(validateHostname('sub.example.com'), true);
		assert.strictEqual(validateHostname('127.0.0.1'), true);
		assert.strictEqual(validateHostname('192.168.1.1'), true);
		assert.strictEqual(validateHostname(''), false);
		assert.strictEqual(validateHostname('   '), false);
		assert.strictEqual(validateHostname(123), false);
	});

	test('getRemediationMessage should provide remediation for serverHost', () => {
		const message = getRemediationMessage('serverHost', '', 'string');
		assert.ok(message.includes('serverHost'));
		assert.ok(message.includes('localhost'));
	});

	test('getRemediationMessage should provide remediation for serverPort', () => {
		const message = getRemediationMessage('serverPort', 0, 'number');
		assert.ok(message.includes('serverPort'));
		assert.ok(message.includes('9000'));
	});

	test('getRemediationMessage should provide remediation for requestTimeout', () => {
		const message = getRemediationMessage('requestTimeout', 50, 'number');
		assert.ok(message.includes('requestTimeout'));
		assert.ok(message.includes('5000'));
	});

	test('getRemediationMessage should provide remediation for providerSelection', () => {
		const message = getRemediationMessage('providerSelection', 'invalid', 'string');
		assert.ok(message.includes('providerSelection'));
		assert.ok(message.includes('lsp-first'));
	});

	test('getRemediationMessage should provide remediation for logLevel', () => {
		const message = getRemediationMessage('logLevel', 'invalid', 'string');
		assert.ok(message.includes('logLevel'));
		assert.ok(message.includes('error'));
	});

	test('getErrorMessage should provide error for serverHost', () => {
		const message = getErrorMessage('serverHost', '', 'string');
		assert.ok(message.includes('Server host'));
		assert.ok(message.includes('non-empty string'));
	});

	test('getErrorMessage should provide error for serverPort', () => {
		const message = getErrorMessage('serverPort', 0, 'number');
		assert.ok(message.includes('Server port'));
		assert.ok(message.includes('1 and 65535'));
	});

	test('getErrorMessage should provide error for requestTimeout', () => {
		const message = getErrorMessage('requestTimeout', 50, 'number');
		assert.ok(message.includes('Request timeout'));
		assert.ok(message.includes('100 and 300000'));
	});

	test('getErrorMessage should provide error for providerSelection', () => {
		const message = getErrorMessage('providerSelection', 'invalid', 'string');
		assert.ok(message.includes('Provider selection'));
		assert.ok(message.includes('lsp-first'));
	});

	test('getErrorMessage should provide error for logLevel', () => {
		const message = getErrorMessage('logLevel', 'invalid', 'string');
		assert.ok(message.includes('Log level'));
		assert.ok(message.includes('error'));
	});

	test('findValidationRule should find rules by field', () => {
		const rule = findValidationRule('serverPort');
		assert.ok(rule);
		assert.strictEqual(rule?.field, 'serverPort');
	});

	test('findValidationRule should return undefined for unknown field', () => {
		const rule = findValidationRule('unknownField');
		assert.strictEqual(rule, undefined);
	});

	test('VALIDATION_RULES should have all required fields', () => {
		const requiredFields = [
			'enabled',
			'serverHost',
			'serverPort',
			'requestTimeout',
			'providerSelection',
			'completionEnabled',
			'diagnosticsEnabled',
			'hoverEnabled',
			'debugMode',
			'logLevel',
		];

		for (const field of requiredFields) {
			const rule = findValidationRule(field);
			assert.ok(rule, `Missing validation rule for ${field}`);
		}
	});

	test('VALIDATION_RULES should validate correctly', () => {
		for (const rule of VALIDATION_RULES) {
			// Each rule should have required properties
			assert.ok(rule.field);
			assert.ok(rule.validate);
			assert.ok(rule.errorMessage);
			assert.ok(rule.remediation);

			// Validate function should be callable
			assert.strictEqual(typeof rule.validate, 'function');
		}
	});
});
