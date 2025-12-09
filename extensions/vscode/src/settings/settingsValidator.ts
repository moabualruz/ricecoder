/**
 * Settings Validator
 * 
 * Provides detailed validation for RiceCoder settings with clear error messages
 * and remediation steps.
 */

export interface ValidationRule {
	field: string;
	validate: (value: unknown) => boolean;
	errorMessage: string;
	remediation: string;
}

export interface ValidationContext {
	field: string;
	value: unknown;
	type: string;
	constraints?: Record<string, unknown>;
}

/**
 * Validate a value against a type
 */
export function validateType(value: unknown, expectedType: string): boolean {
	switch (expectedType) {
		case 'string':
			return typeof value === 'string';
		case 'number':
			return typeof value === 'number' && !isNaN(value);
		case 'boolean':
			return typeof value === 'boolean';
		case 'integer':
			return Number.isInteger(value);
		case 'array':
			return Array.isArray(value);
		case 'object':
			return typeof value === 'object' && value !== null && !Array.isArray(value);
		default:
			return false;
	}
}

/**
 * Validate a number is within range
 */
export function validateNumberRange(value: unknown, min: number, max: number): boolean {
	if (typeof value !== 'number' || isNaN(value)) {
		return false;
	}
	return value >= min && value <= max;
}

/**
 * Validate a string is not empty
 */
export function validateNonEmptyString(value: unknown): boolean {
	return typeof value === 'string' && value.trim().length > 0;
}

/**
 * Validate a value is in an enum
 */
export function validateEnum(value: unknown, allowedValues: unknown[]): boolean {
	return allowedValues.includes(value);
}

/**
 * Validate a port number
 */
export function validatePort(value: unknown): boolean {
	return validateNumberRange(value, 1, 65535) && Number.isInteger(value);
}

/**
 * Validate a timeout value
 */
export function validateTimeout(value: unknown): boolean {
	return validateNumberRange(value, 100, 300000) && Number.isInteger(value);
}

/**
 * Validate a hostname or IP address
 */
export function validateHostname(value: unknown): boolean {
	if (!validateNonEmptyString(value)) {
		return false;
	}

	const str = value as string;

	// Allow localhost, IP addresses, and hostnames
	const hostnameRegex = /^([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$|^localhost$|^(\d{1,3}\.){3}\d{1,3}$/;

	return hostnameRegex.test(str);
}

/**
 * Get remediation message for a validation error
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function getRemediationMessage(field: string, _value: unknown, _expectedType: string, _constraints?: Record<string, unknown>): string {
	switch (field) {
		case 'serverHost':
			return 'Set ricecoder.serverHost to a valid hostname or IP address (e.g., "localhost" or "127.0.0.1")';

		case 'serverPort':
			return `Set ricecoder.serverPort to a valid port number between 1 and 65535 (e.g., 9000)`;

		case 'requestTimeout':
			return `Set ricecoder.requestTimeout to a value between 100 and 300000 milliseconds (e.g., 5000 for 5 seconds)`;

		case 'providerSelection':
			return `Set ricecoder.providerSelection to one of: lsp-first, configured-rules, builtin, generic`;

		case 'logLevel':
			return `Set ricecoder.logLevel to one of: error, warn, info, debug`;

		case 'enabled':
		case 'completionEnabled':
		case 'diagnosticsEnabled':
		case 'hoverEnabled':
		case 'debugMode':
			return `Set ricecoder.${field} to true or false`;

		default:
			return `Fix the value for ricecoder.${field}`;
	}
}

/**
 * Get error message for a validation error
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function getErrorMessage(field: string, value: unknown, _expectedType: string, _constraints?: Record<string, unknown>): string {
	switch (field) {
		case 'serverHost':
			return `Server host must be a non-empty string (got: ${JSON.stringify(value)})`;

		case 'serverPort':
			return `Server port must be an integer between 1 and 65535 (got: ${JSON.stringify(value)})`;

		case 'requestTimeout':
			return `Request timeout must be an integer between 100 and 300000 milliseconds (got: ${JSON.stringify(value)})`;

		case 'providerSelection':
			return `Provider selection must be one of: lsp-first, configured-rules, builtin, generic (got: ${JSON.stringify(value)})`;

		case 'logLevel':
			return `Log level must be one of: error, warn, info, debug (got: ${JSON.stringify(value)})`;

		case 'enabled':
		case 'completionEnabled':
		case 'diagnosticsEnabled':
		case 'hoverEnabled':
		case 'debugMode':
			return `${field} must be a boolean (got: ${JSON.stringify(value)})`;

		default:
			return `Invalid value for ricecoder.${field}: ${JSON.stringify(value)}`;
	}
}

/**
 * Validation rules for RiceCoder settings
 */
export const VALIDATION_RULES: ValidationRule[] = [
	{
		field: 'enabled',
		validate: (value) => typeof value === 'boolean',
		errorMessage: 'enabled must be a boolean',
		remediation: 'Set ricecoder.enabled to true or false',
	},
	{
		field: 'serverHost',
		validate: (value) => validateHostname(value),
		errorMessage: 'serverHost must be a valid hostname or IP address',
		remediation: 'Set ricecoder.serverHost to a valid hostname or IP address (e.g., "localhost" or "127.0.0.1")',
	},
	{
		field: 'serverPort',
		validate: (value) => validatePort(value),
		errorMessage: 'serverPort must be an integer between 1 and 65535',
		remediation: 'Set ricecoder.serverPort to a valid port number (e.g., 9000)',
	},
	{
		field: 'requestTimeout',
		validate: (value) => validateTimeout(value),
		errorMessage: 'requestTimeout must be an integer between 100 and 300000 milliseconds',
		remediation: 'Set ricecoder.requestTimeout to a value between 100 and 300000 (e.g., 5000 for 5 seconds)',
	},
	{
		field: 'providerSelection',
		validate: (value) => validateEnum(value, ['lsp-first', 'configured-rules', 'builtin', 'generic']),
		errorMessage: 'providerSelection must be one of: lsp-first, configured-rules, builtin, generic',
		remediation: 'Set ricecoder.providerSelection to one of: lsp-first, configured-rules, builtin, generic',
	},
	{
		field: 'completionEnabled',
		validate: (value) => typeof value === 'boolean',
		errorMessage: 'completionEnabled must be a boolean',
		remediation: 'Set ricecoder.completionEnabled to true or false',
	},
	{
		field: 'diagnosticsEnabled',
		validate: (value) => typeof value === 'boolean',
		errorMessage: 'diagnosticsEnabled must be a boolean',
		remediation: 'Set ricecoder.diagnosticsEnabled to true or false',
	},
	{
		field: 'hoverEnabled',
		validate: (value) => typeof value === 'boolean',
		errorMessage: 'hoverEnabled must be a boolean',
		remediation: 'Set ricecoder.hoverEnabled to true or false',
	},
	{
		field: 'debugMode',
		validate: (value) => typeof value === 'boolean',
		errorMessage: 'debugMode must be a boolean',
		remediation: 'Set ricecoder.debugMode to true or false',
	},
	{
		field: 'logLevel',
		validate: (value) => validateEnum(value, ['error', 'warn', 'info', 'debug']),
		errorMessage: 'logLevel must be one of: error, warn, info, debug',
		remediation: 'Set ricecoder.logLevel to one of: error, warn, info, debug',
	},
];

/**
 * Find validation rule for a field
 */
export function findValidationRule(field: string): ValidationRule | undefined {
	return VALIDATION_RULES.find((rule) => rule.field === field);
}
