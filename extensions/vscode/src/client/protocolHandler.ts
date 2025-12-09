/**
 * Protocol handler for VS Code communication with RiceCoder
 * 
 * Manages:
 * 1. Request/response protocol
 * 2. Streaming protocol
 * 3. Notification protocol
 * 4. Error handling and recovery
 * 5. Protocol versioning
 */

/**
 * Protocol version
 */
export const PROTOCOL_VERSION = '1.0.0';

/**
 * Request types
 */
export enum RequestType {
	// IDE Features
	Completion = 'completion/provide',
	ResolveCompletion = 'completion/resolve',
	Diagnostics = 'diagnostics/provide',
	Hover = 'hover/provide',
	Definition = 'definition/provide',

	// Commands
	Chat = 'chat/send',
	Review = 'review/code',
	Generate = 'generate/code',
	Refactor = 'refactor/code',

	// Streaming
	StreamStart = 'stream/start',
	StreamCancel = 'stream/cancel',

	// Configuration
	GetConfig = 'config/get',
	SetConfig = 'config/set',
	ReloadConfig = 'config/reload'
}

/**
 * Notification types
 */
export enum NotificationType {
	// Server notifications
	ServerReady = 'server/ready',
	ServerShutdown = 'server/shutdown',
	ServerError = 'server/error',

	// Configuration notifications
	ConfigChanged = 'config/changed',
	ProviderChanged = 'provider/changed',

	// Stream notifications
	StreamChunk = 'stream/chunk',
	StreamComplete = 'stream/complete',
	StreamError = 'stream/error'
}

/**
 * Request parameters for IDE features
 */
export interface CompletionParams {
	language: string;
	file_path: string;
	position: Position;
	context: string;
	trigger_character?: string;
}

export interface DiagnosticsParams {
	language: string;
	file_path: string;
	source: string;
}

export interface HoverParams {
	language: string;
	file_path: string;
	position: Position;
	word: string;
}

export interface DefinitionParams {
	language: string;
	file_path: string;
	position: Position;
}

/**
 * Response types for IDE features
 */
export interface CompletionItem {
	label: string;
	kind: string;
	detail?: string;
	documentation?: string;
	insert_text: string;
	sort_text?: string;
	filter_text?: string;
}

export interface Diagnostic {
	range: Range;
	severity: 'Error' | 'Warning' | 'Information' | 'Hint';
	message: string;
	source: string;
	code?: string;
	quick_fixes?: QuickFix[];
}

export interface Hover {
	contents: string;
	type_info?: string;
	documentation?: string;
	source?: string;
	range?: Range;
}

export interface Location {
	file_path: string;
	range: Range;
}

/**
 * Common types
 */
export interface Position {
	line: number;
	character: number;
}

export interface Range {
	start: Position;
	end: Position;
}

export interface QuickFix {
	title: string;
	kind: string;
	edit: TextEdit;
}

export interface TextEdit {
	range: Range;
	new_text: string;
}

/**
 * Command request parameters
 */
export interface ChatParams {
	message: string;
	context: string;
	language: string;
	file_path: string;
}

export interface ReviewParams {
	language: string;
	file_path: string;
	source: string;
}

export interface GenerateParams {
	prompt: string;
	language: string;
	file_path: string;
	context: string;
}

export interface RefactorParams {
	refactoring_type: string;
	language: string;
	file_path: string;
	source: string;
}

/**
 * Command response types
 */
export interface ChatResponse {
	response: string;
	stream_id?: string;
}

export interface ReviewResponse {
	review: string;
	issues: ReviewIssue[];
}

export interface ReviewIssue {
	line: number;
	severity: 'Error' | 'Warning' | 'Info';
	message: string;
	suggestion?: string;
}

export interface GenerateResponse {
	code: string;
	explanation?: string;
}

export interface RefactorResponse {
	refactored: string;
	changes: Change[];
}

export interface Change {
	type: 'add' | 'remove' | 'modify';
	range: Range;
	old_text?: string;
	new_text?: string;
}

/**
 * Configuration types
 */
export interface IdeConfig {
	enabled: boolean;
	serverHost: string;
	serverPort: number;
	requestTimeout: number;
	completionEnabled: boolean;
	diagnosticsEnabled: boolean;
	hoverEnabled: boolean;
	providerSelection: 'lsp-first' | 'configured-rules' | 'builtin' | 'generic';
}

/**
 * Stream types
 */
export interface StreamStartParams {
	method: string;
	params?: unknown;
	stream_id: string;
}

export interface StreamChunk {
	stream_id: string;
	chunk: unknown;
	index: number;
	total?: number;
}

export interface StreamComplete {
	stream_id: string;
	chunks: unknown[];
}

export interface StreamError {
	stream_id: string;
	error: string;
	code?: number;
}

/**
 * Error types
 */
export interface ProtocolError {
	code: number;
	message: string;
	data?: unknown;
}

export enum ErrorCode {
	ParseError = -32700,
	InvalidRequest = -32600,
	MethodNotFound = -32601,
	InvalidParams = -32602,
	InternalError = -32603,
	ServerErrorStart = -32099,
	ServerErrorEnd = -32000,
	ServerNotInitialized = -32002,
	UnknownErrorCode = -32001,

	// Custom error codes
	ConnectionError = 1000,
	TimeoutError = 1001,
	StreamError = 1002,
	ConfigError = 1003
}

/**
 * Protocol helper functions
 */
export function isValidPosition(pos: unknown): pos is Position {
	if (typeof pos !== 'object' || pos === null) {
		return false;
	}
	const p = pos as Record<string, unknown>;
	return typeof p.line === 'number' && typeof p.character === 'number';
}

export function isValidRange(range: unknown): range is Range {
	if (typeof range !== 'object' || range === null) {
		return false;
	}
	const r = range as Record<string, unknown>;
	return isValidPosition(r.start) && isValidPosition(r.end);
}

export function createErrorResponse(code: number, message: string, data?: unknown): ProtocolError {
	return { code, message, data };
}

export function isErrorResponse(response: unknown): response is ProtocolError {
	if (typeof response !== 'object' || response === null) {
		return false;
	}
	const r = response as Record<string, unknown>;
	return typeof r.code === 'number' && typeof r.message === 'string';
}
