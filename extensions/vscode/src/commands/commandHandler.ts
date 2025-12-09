import * as vscode from 'vscode';
import { RicecoderClient } from '../client/ricecoderClient';

/**
 * Command handler for RiceCoder commands
 * 
 * Registers and handles:
 * 1. Chat command - Open chat interface
 * 2. Review command - Review code
 * 3. Generate command - Generate code
 * 4. Refactor command - Refactor code
 */
export class CommandHandler {
	constructor(private client: RicecoderClient) {}

	/**
	 * Register all RiceCoder commands
	 */
	registerCommands(context: vscode.ExtensionContext): void {
		// Chat command
		const chatCommand = vscode.commands.registerCommand('ricecoder.chat', () => {
			this.handleChatCommand();
		});
		context.subscriptions.push(chatCommand);

		// Review command
		const reviewCommand = vscode.commands.registerCommand('ricecoder.review', () => {
			this.handleReviewCommand();
		});
		context.subscriptions.push(reviewCommand);

		// Generate command
		const generateCommand = vscode.commands.registerCommand('ricecoder.generate', () => {
			this.handleGenerateCommand();
		});
		context.subscriptions.push(generateCommand);

		// Refactor command
		const refactorCommand = vscode.commands.registerCommand('ricecoder.refactor', () => {
			this.handleRefactorCommand();
		});
		context.subscriptions.push(refactorCommand);
	}

	/**
	 * Handle chat command
	 */
	private async handleChatCommand(): Promise<void> {
		try {
			const editor = vscode.window.activeTextEditor;
			if (!editor) {
				vscode.window.showWarningMessage('No active editor');
				return;
			}

			// Get selected text or current line
			const selection = editor.selection;
			const selectedText = editor.document.getText(selection);
			const context = selectedText || editor.document.getText(
				new vscode.Range(
					new vscode.Position(selection.active.line, 0),
					new vscode.Position(selection.active.line + 1, 0)
				)
			);

			// Show input box for chat message
			const message = await vscode.window.showInputBox({
				prompt: 'Enter your message for RiceCoder',
				placeHolder: 'Ask RiceCoder...'
			});

			if (!message) {
				return;
			}

			// Send chat request
			const params = {
				message,
				context,
				language: editor.document.languageId,
				file_path: editor.document.uri.fsPath
			};

			const result = await this.client.request('chat/send', params);
			const response = result as Record<string, unknown>;

			// Display response
			vscode.window.showInformationMessage(String(response.response || 'No response'));
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			vscode.window.showErrorMessage(`Chat command failed: ${message}`);
		}
	}

	/**
	 * Handle review command
	 */
	private async handleReviewCommand(): Promise<void> {
		try {
			const editor = vscode.window.activeTextEditor;
			if (!editor) {
				vscode.window.showWarningMessage('No active editor');
				return;
			}

			// Show progress
			await vscode.window.withProgress(
				{
					location: vscode.ProgressLocation.Notification,
					title: 'RiceCoder: Reviewing code...',
					cancellable: false
				},
				async () => {
					// Send review request
					const params = {
						language: editor.document.languageId,
						file_path: editor.document.uri.fsPath,
						source: editor.document.getText()
					};

					const result = await this.client.request('review/code', params);
					const response = result as Record<string, unknown>;

					// Display review results
					const review = String(response.review || 'No review available');
					vscode.window.showInformationMessage(`Review: ${review}`);
				}
			);
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			vscode.window.showErrorMessage(`Review command failed: ${message}`);
		}
	}

	/**
	 * Handle generate command
	 */
	private async handleGenerateCommand(): Promise<void> {
		try {
			const editor = vscode.window.activeTextEditor;
			if (!editor) {
				vscode.window.showWarningMessage('No active editor');
				return;
			}

			// Show input box for generation prompt
			const prompt = await vscode.window.showInputBox({
				prompt: 'Describe what you want to generate',
				placeHolder: 'Generate...'
			});

			if (!prompt) {
				return;
			}

			// Show progress
			await vscode.window.withProgress(
				{
					location: vscode.ProgressLocation.Notification,
					title: 'RiceCoder: Generating code...',
					cancellable: false
				},
				async () => {
					// Send generate request
					const params = {
						prompt,
						language: editor.document.languageId,
						file_path: editor.document.uri.fsPath,
						context: editor.document.getText(
							new vscode.Range(
								new vscode.Position(Math.max(0, editor.selection.active.line - 10), 0),
								new vscode.Position(editor.selection.active.line + 10, 0)
							)
						)
					};

					const result = await this.client.request('generate/code', params);
					const response = result as Record<string, unknown>;

					// Insert generated code
					const generated = String(response.code || '');
					if (generated) {
						editor.edit((editBuilder) => {
							editBuilder.insert(editor.selection.active, generated);
						});
					}
				}
			);
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			vscode.window.showErrorMessage(`Generate command failed: ${message}`);
		}
	}

	/**
	 * Handle refactor command
	 */
	private async handleRefactorCommand(): Promise<void> {
		try {
			const editor = vscode.window.activeTextEditor;
			if (!editor) {
				vscode.window.showWarningMessage('No active editor');
				return;
			}

			// Get selected text
			const selection = editor.selection;
			const selectedText = editor.document.getText(selection);

			if (!selectedText) {
				vscode.window.showWarningMessage('Please select code to refactor');
				return;
			}

			// Show refactoring options
			const refactoringType = await vscode.window.showQuickPick(
				[
					'Extract Function',
					'Rename Variable',
					'Simplify Logic',
					'Optimize Performance',
					'Add Documentation'
				],
				{ placeHolder: 'Select refactoring type' }
			);

			if (!refactoringType) {
				return;
			}

			// Show progress
			await vscode.window.withProgress(
				{
					location: vscode.ProgressLocation.Notification,
					title: `RiceCoder: ${refactoringType}...`,
					cancellable: false
				},
				async () => {
					// Send refactor request
					const params = {
						refactoring_type: refactoringType,
						language: editor.document.languageId,
						file_path: editor.document.uri.fsPath,
						source: selectedText
					};

					const result = await this.client.request('refactor/code', params);
					const response = result as Record<string, unknown>;

					// Replace selected text with refactored code
					const refactored = String(response.refactored || '');
					if (refactored) {
						editor.edit((editBuilder) => {
							editBuilder.replace(selection, refactored);
						});
					}
				}
			);
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			vscode.window.showErrorMessage(`Refactor command failed: ${message}`);
		}
	}
}
