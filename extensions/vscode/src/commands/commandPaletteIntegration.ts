import * as vscode from 'vscode';

/**
 * Command Palette Integration
 * 
 * Provides enhanced command palette integration for RiceCoder commands:
 * - Command grouping under "RiceCoder" category
 * - Keyboard shortcuts for quick access
 * - Context menu integration for editor
 * - Command descriptions and help text
 * 
 * Keyboard Shortcuts:
 * - Ctrl+Shift+R (Cmd+Shift+R on Mac): Chat with RiceCoder
 * - Ctrl+Shift+E (Cmd+Shift+E on Mac): Review Code
 * - Ctrl+Shift+G (Cmd+Shift+G on Mac): Generate Code
 * - Ctrl+Shift+F (Cmd+Shift+F on Mac): Refactor Code
 */

/**
 * Command palette command definitions
 * These are registered in package.json and provide:
 * - Command ID (ricecoder.*)
 * - Display title for command palette
 * - Category for grouping
 * - Description for help text
 */
export const RICECODER_COMMANDS = {
	CHAT: {
		id: 'ricecoder.chat',
		title: 'Chat with RiceCoder',
		category: 'RiceCoder',
		description: 'Open chat interface to ask RiceCoder questions about your code',
		keybinding: {
			windows: 'ctrl+shift+r',
			mac: 'cmd+shift+r'
		}
	},
	REVIEW: {
		id: 'ricecoder.review',
		title: 'Review Code',
		category: 'RiceCoder',
		description: 'Get AI-powered code review and suggestions for the current file',
		keybinding: {
			windows: 'ctrl+shift+e',
			mac: 'cmd+shift+e'
		}
	},
	GENERATE: {
		id: 'ricecoder.generate',
		title: 'Generate Code',
		category: 'RiceCoder',
		description: 'Generate code based on your description',
		keybinding: {
			windows: 'ctrl+shift+g',
			mac: 'cmd+shift+g'
		}
	},
	REFACTOR: {
		id: 'ricecoder.refactor',
		title: 'Refactor Code',
		category: 'RiceCoder',
		description: 'Refactor selected code with various refactoring options',
		keybinding: {
			windows: 'ctrl+shift+f',
			mac: 'cmd+shift+f'
		}
	}
};

/**
 * Get all available commands for display
 * 
 * Returns a list of all RiceCoder commands with their metadata
 * for use in command palette and help systems
 */
export function getAvailableCommands(): typeof RICECODER_COMMANDS[keyof typeof RICECODER_COMMANDS][] {
	return Object.values(RICECODER_COMMANDS);
}

/**
 * Get command by ID
 * 
 * @param commandId - The command ID (e.g., 'ricecoder.chat')
 * @returns The command metadata or undefined if not found
 */
export function getCommandById(commandId: string): typeof RICECODER_COMMANDS[keyof typeof RICECODER_COMMANDS] | undefined {
	return Object.values(RICECODER_COMMANDS).find(cmd => cmd.id === commandId);
}

/**
 * Show command palette help
 * 
 * Displays a quick pick menu showing all available RiceCoder commands
 * with their keyboard shortcuts and descriptions
 */
export async function showCommandPaletteHelp(): Promise<void> {
	const commands = getAvailableCommands();
	
	const items = commands.map(cmd => ({
		label: `$(symbol-method) ${cmd.title}`,
		description: cmd.description,
		detail: `Keyboard: ${cmd.keybinding.windows} (${cmd.keybinding.mac} on Mac)`,
		commandId: cmd.id
	}));

	const selected = await vscode.window.showQuickPick(items, {
		placeHolder: 'Select a RiceCoder command',
		matchOnDescription: true,
		matchOnDetail: true
	});

	if (selected) {
		await vscode.commands.executeCommand(selected.commandId);
	}
}

/**
 * Register context menu items
 * 
 * Adds RiceCoder commands to the editor context menu
 * for easy access when right-clicking in the editor
 * 
 * Note: Context menu items are defined in package.json under menus.editor/context
 * This function is a placeholder for future context menu enhancements
 */
export function registerContextMenuItems(): void {
	// These are defined in package.json under menus.editor/context
	// This function is a placeholder for future context menu enhancements
}

/**
 * Get keyboard shortcut for command
 * 
 * @param commandId - The command ID
 * @returns The keyboard shortcut string or undefined
 */
export function getKeyboardShortcut(commandId: string): string | undefined {
	const command = getCommandById(commandId);
	if (!command) {
		return undefined;
	}

	const isMac = process.platform === 'darwin';
	return isMac ? command.keybinding.mac : command.keybinding.windows;
}

/**
 * Show keyboard shortcuts help
 * 
 * Displays all available keyboard shortcuts for RiceCoder commands
 */
export async function showKeyboardShortcutsHelp(): Promise<void> {
	const commands = getAvailableCommands();
	
	const shortcuts = commands
		.map(cmd => `${cmd.title}: ${cmd.keybinding.windows} (${cmd.keybinding.mac} on Mac)`)
		.join('\n');

	const message = `RiceCoder Keyboard Shortcuts:\n\n${shortcuts}`;
	
	await vscode.window.showInformationMessage(message, { modal: true });
}
