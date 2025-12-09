import * as net from 'net';
import { EventEmitter } from 'events';

/**
 * JSON-RPC request/response types
 */
interface JsonRpcRequest {
	jsonrpc: '2.0';
	id: number;
	method: string;
	params?: unknown;
}

interface JsonRpcResponse {
	jsonrpc: '2.0';
	id: number;
	result?: unknown;
	error?: {
		code: number;
		message: string;
		data?: unknown;
	};
}

interface JsonRpcNotification {
	jsonrpc: '2.0';
	method: string;
	params?: unknown;
}

/**
 * RiceCoder client for JSON-RPC communication with the backend
 * 
 * Handles:
 * - Connection management (connect/disconnect)
 * - Request/response handling with timeouts
 * - Streaming responses for long-running operations
 * - Error handling and reconnection
 */
export class RicecoderClient extends EventEmitter {
	private socket: net.Socket | null = null;
	private host: string;
	private port: number;
	private requestTimeout: number;
	private requestId: number = 0;
	private pendingRequests: Map<number, {
		resolve: (value: unknown) => void;
		reject: (error: Error) => void;
		timeout: NodeJS.Timeout;
	}> = new Map();
	private buffer: string = '';
	private connected: boolean = false;

	constructor(host: string, port: number, requestTimeout: number = 5000) {
		super();
		this.host = host;
		this.port = port;
		this.requestTimeout = requestTimeout;
	}

	/**
	 * Connect to the RiceCoder server
	 */
	async connect(): Promise<void> {
		return new Promise((resolve, reject) => {
			try {
				this.socket = net.createConnection(this.port, this.host);

				this.socket.on('connect', () => {
					this.connected = true;
					console.log(`Connected to RiceCoder server at ${this.host}:${this.port}`);
					resolve();
				});

				this.socket.on('data', (data) => {
					this.handleData(data);
				});

				this.socket.on('error', (error) => {
					this.connected = false;
					console.error('Socket error:', error);
					this.emit('error', error);
					reject(error);
				});

				this.socket.on('close', () => {
					this.connected = false;
					console.log('Disconnected from RiceCoder server');
					this.emit('disconnected');
				});

				// Set connection timeout
				this.socket.setTimeout(10000, () => {
					this.socket?.destroy();
					reject(new Error('Connection timeout'));
				});
			} catch (error) {
				reject(error);
			}
		});
	}

	/**
	 * Disconnect from the RiceCoder server
	 */
	async disconnect(): Promise<void> {
		return new Promise((resolve) => {
			if (this.socket) {
				this.socket.end(() => {
					this.socket = null;
					this.connected = false;
					resolve();
				});
			} else {
				resolve();
			}
		});
	}

	/**
	 * Send a JSON-RPC request and wait for response
	 */
	async request(method: string, params?: unknown): Promise<unknown> {
		if (!this.connected || !this.socket) {
			throw new Error('Not connected to RiceCoder server');
		}

		const id = ++this.requestId;
		const request: JsonRpcRequest = {
			jsonrpc: '2.0',
			id,
			method,
			params
		};

		return new Promise((resolve, reject) => {
			const timeout = setTimeout(() => {
				this.pendingRequests.delete(id);
				reject(new Error(`Request timeout for method: ${method}`));
			}, this.requestTimeout);

			this.pendingRequests.set(id, { resolve, reject, timeout });

			try {
				const message = JSON.stringify(request) + '\n';
				this.socket!.write(message, (error) => {
					if (error) {
						clearTimeout(timeout);
						this.pendingRequests.delete(id);
						reject(error);
					}
				});
			} catch (error) {
				clearTimeout(timeout);
				this.pendingRequests.delete(id);
				reject(error);
			}
		});
	}

	/**
	 * Send a JSON-RPC notification (no response expected)
	 */
	notify(method: string, params?: unknown): void {
		if (!this.connected || !this.socket) {
			console.warn('Not connected to RiceCoder server, notification not sent');
			return;
		}

		const notification: JsonRpcNotification = {
			jsonrpc: '2.0',
			method,
			params
		};

		try {
			const message = JSON.stringify(notification) + '\n';
			this.socket.write(message);
		} catch (error) {
			console.error('Failed to send notification:', error);
		}
	}

	/**
	 * Handle incoming data from the server
	 */
	private handleData(data: Buffer): void {
		this.buffer += data.toString('utf-8');

		// Process complete messages (delimited by newlines)
		const lines = this.buffer.split('\n');
		this.buffer = lines.pop() || ''; // Keep incomplete line in buffer

		for (const line of lines) {
			if (line.trim()) {
				try {
					const message = JSON.parse(line) as JsonRpcResponse;
					this.handleMessage(message);
				} catch (error) {
					console.error('Failed to parse message:', error, 'Line:', line);
				}
			}
		}
	}

	/**
	 * Handle a JSON-RPC response
	 */
	private handleMessage(message: JsonRpcResponse): void {
		const { id, result, error } = message;

		const pending = this.pendingRequests.get(id);
		if (!pending) {
			console.warn(`Received response for unknown request id: ${id}`);
			return;
		}

		this.pendingRequests.delete(id);
		clearTimeout(pending.timeout);

		if (error) {
			pending.reject(new Error(`${error.message} (code: ${error.code})`));
		} else {
			pending.resolve(result);
		}
	}

	/**
	 * Check if connected to server
	 */
	isConnected(): boolean {
		return this.connected;
	}

	/**
	 * Get server host
	 */
	getHost(): string {
		return this.host;
	}

	/**
	 * Get server port
	 */
	getPort(): number {
		return this.port;
	}

	/**
	 * Set request timeout
	 */
	setRequestTimeout(timeout: number): void {
		this.requestTimeout = timeout;
	}
}
