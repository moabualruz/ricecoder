import { EventEmitter } from 'events';
import { RicecoderClient } from './ricecoderClient';

/**
 * Streaming response handler for long-running operations
 * 
 * Handles streaming responses from RiceCoder backend by:
 * 1. Initiating streaming requests
 * 2. Collecting streamed chunks
 * 3. Emitting events for each chunk
 * 4. Handling stream completion and errors
 */
export class StreamingHandler extends EventEmitter {
	private activeStreams: Map<string, StreamContext> = new Map();
	private streamId: number = 0;

	constructor(private client: RicecoderClient) {
		super();
	}

	/**
	 * Start a streaming request
	 * 
	 * Returns a stream ID that can be used to track the stream
	 */
	async startStream(method: string, params?: unknown): Promise<string> {
		const streamId = `stream_${++this.streamId}`;
		
		const context: StreamContext = {
			id: streamId,
			method,
			params,
			chunks: [],
			completed: false,
			error: null
		};

		this.activeStreams.set(streamId, context);

		try {
			// Send streaming request to backend
			const requestParams = params && typeof params === 'object'
				? { ...(params as Record<string, unknown>), stream_id: streamId }
				: { stream_id: streamId };
			const result = await this.client.request(`${method}/stream`, requestParams);

			// Mark stream as started
			this.emit('stream:start', { streamId, result });
		} catch (error) {
			context.error = error instanceof Error ? error : new Error(String(error));
			this.emit('stream:error', { streamId, error: context.error });
			this.activeStreams.delete(streamId);
			throw error;
		}

		return streamId;
	}

	/**
	 * Add a chunk to a stream
	 * 
	 * Called by the extension when receiving streamed data
	 */
	addChunk(streamId: string, chunk: unknown): void {
		const context = this.activeStreams.get(streamId);
		if (!context) {
			console.warn(`Received chunk for unknown stream: ${streamId}`);
			return;
		}

		context.chunks.push(chunk);
		this.emit('stream:chunk', { streamId, chunk });
	}

	/**
	 * Complete a stream
	 * 
	 * Called when the stream is finished
	 */
	completeStream(streamId: string): void {
		const context = this.activeStreams.get(streamId);
		if (!context) {
			console.warn(`Completed unknown stream: ${streamId}`);
			return;
		}

		context.completed = true;
		this.emit('stream:complete', { streamId, chunks: context.chunks });
		this.activeStreams.delete(streamId);
	}

	/**
	 * Error a stream
	 * 
	 * Called when the stream encounters an error
	 */
	errorStream(streamId: string, error: Error): void {
		const context = this.activeStreams.get(streamId);
		if (!context) {
			console.warn(`Error on unknown stream: ${streamId}`);
			return;
		}

		context.error = error;
		this.emit('stream:error', { streamId, error });
		this.activeStreams.delete(streamId);
	}

	/**
	 * Get all chunks from a stream
	 */
	getChunks(streamId: string): unknown[] {
		const context = this.activeStreams.get(streamId);
		return context ? context.chunks : [];
	}

	/**
	 * Get combined text from all chunks
	 */
	getCombinedText(streamId: string): string {
		const chunks = this.getChunks(streamId);
		return chunks.map(chunk => {
			if (typeof chunk === 'string') {
				return chunk;
			}
			if (typeof chunk === 'object' && chunk !== null && 'text' in chunk) {
				return String((chunk as Record<string, unknown>).text);
			}
			return String(chunk);
		}).join('');
	}

	/**
	 * Check if a stream is active
	 */
	isStreamActive(streamId: string): boolean {
		const context = this.activeStreams.get(streamId);
		return context ? !context.completed && !context.error : false;
	}

	/**
	 * Cancel a stream
	 */
	async cancelStream(streamId: string): Promise<void> {
		const context = this.activeStreams.get(streamId);
		if (!context) {
			return;
		}

		try {
			await this.client.request('stream/cancel', { stream_id: streamId });
		} catch (error) {
			console.error(`Failed to cancel stream ${streamId}:`, error);
		}

		this.activeStreams.delete(streamId);
		this.emit('stream:cancelled', { streamId });
	}

	/**
	 * Listen for stream events
	 */
	onStreamChunk(streamId: string, callback: (chunk: unknown) => void): void {
		const handler = (data: { streamId: string; chunk: unknown }) => {
			if (data.streamId === streamId) {
				callback(data.chunk);
			}
		};
		this.on('stream:chunk', handler);
	}

	/**
	 * Listen for stream completion
	 */
	onStreamComplete(streamId: string, callback: (chunks: unknown[]) => void): void {
		const handler = (data: { streamId: string; chunks: unknown[] }) => {
			if (data.streamId === streamId) {
				callback(data.chunks);
				this.removeListener('stream:complete', handler);
			}
		};
		this.on('stream:complete', handler);
	}

	/**
	 * Listen for stream errors
	 */
	onStreamError(streamId: string, callback: (error: Error) => void): void {
		const handler = (data: { streamId: string; error: Error }) => {
			if (data.streamId === streamId) {
				callback(data.error);
				this.removeListener('stream:error', handler);
			}
		};
		this.on('stream:error', handler);
	}
}

/**
 * Stream context for tracking active streams
 */
interface StreamContext {
	id: string;
	method: string;
	params?: unknown;
	chunks: unknown[];
	completed: boolean;
	error: Error | null;
}
