import * as assert from 'assert';
import {
	RICECODER_COMMANDS,
	getAvailableCommands,
	getCommandById,
	getKeyboardShortcut
} from './commandPaletteIntegration';

/**
 * Test suite for command palette integration
 * 
 * Tests verify:
 * - All commands are properly defined
 * - Command metadata is correct
 * - Keyboard shortcuts are accessible
 * - Command lookup functions work correctly
 */

describe('Command Palette Integration', () => {
	describe('RICECODER_COMMANDS', () => {
		it('should have all required commands', () => {
			assert.ok(RICECODER_COMMANDS.CHAT, 'CHAT command should exist');
			assert.ok(RICECODER_COMMANDS.REVIEW, 'REVIEW command should exist');
			assert.ok(RICECODER_COMMANDS.GENERATE, 'GENERATE command should exist');
			assert.ok(RICECODER_COMMANDS.REFACTOR, 'REFACTOR command should exist');
		});

		it('should have correct command IDs', () => {
			assert.strictEqual(RICECODER_COMMANDS.CHAT.id, 'ricecoder.chat');
			assert.strictEqual(RICECODER_COMMANDS.REVIEW.id, 'ricecoder.review');
			assert.strictEqual(RICECODER_COMMANDS.GENERATE.id, 'ricecoder.generate');
			assert.strictEqual(RICECODER_COMMANDS.REFACTOR.id, 'ricecoder.refactor');
		});

		it('should have correct categories', () => {
			assert.strictEqual(RICECODER_COMMANDS.CHAT.category, 'RiceCoder');
			assert.strictEqual(RICECODER_COMMANDS.REVIEW.category, 'RiceCoder');
			assert.strictEqual(RICECODER_COMMANDS.GENERATE.category, 'RiceCoder');
			assert.strictEqual(RICECODER_COMMANDS.REFACTOR.category, 'RiceCoder');
		});

		it('should have descriptions', () => {
			assert.ok(RICECODER_COMMANDS.CHAT.description, 'CHAT should have description');
			assert.ok(RICECODER_COMMANDS.REVIEW.description, 'REVIEW should have description');
			assert.ok(RICECODER_COMMANDS.GENERATE.description, 'GENERATE should have description');
			assert.ok(RICECODER_COMMANDS.REFACTOR.description, 'REFACTOR should have description');
		});

		it('should have keyboard bindings', () => {
			assert.ok(RICECODER_COMMANDS.CHAT.keybinding, 'CHAT should have keybinding');
			assert.ok(RICECODER_COMMANDS.REVIEW.keybinding, 'REVIEW should have keybinding');
			assert.ok(RICECODER_COMMANDS.GENERATE.keybinding, 'GENERATE should have keybinding');
			assert.ok(RICECODER_COMMANDS.REFACTOR.keybinding, 'REFACTOR should have keybinding');
		});

		it('should have Windows and Mac keybindings', () => {
			assert.ok(RICECODER_COMMANDS.CHAT.keybinding.windows, 'CHAT should have Windows keybinding');
			assert.ok(RICECODER_COMMANDS.CHAT.keybinding.mac, 'CHAT should have Mac keybinding');
		});
	});

	describe('getAvailableCommands', () => {
		it('should return all commands', () => {
			const commands = getAvailableCommands();
			assert.strictEqual(commands.length, 4, 'Should return 4 commands');
		});

		it('should return commands with correct structure', () => {
			const commands = getAvailableCommands();
			commands.forEach(cmd => {
				assert.ok(cmd.id, 'Command should have id');
				assert.ok(cmd.title, 'Command should have title');
				assert.ok(cmd.category, 'Command should have category');
				assert.ok(cmd.description, 'Command should have description');
				assert.ok(cmd.keybinding, 'Command should have keybinding');
			});
		});
	});

	describe('getCommandById', () => {
		it('should find command by ID', () => {
			const cmd = getCommandById('ricecoder.chat');
			assert.ok(cmd, 'Should find chat command');
			assert.strictEqual(cmd?.id, 'ricecoder.chat');
		});

		it('should return undefined for unknown command', () => {
			const cmd = getCommandById('ricecoder.unknown');
			assert.strictEqual(cmd, undefined, 'Should return undefined for unknown command');
		});

		it('should find all commands', () => {
			assert.ok(getCommandById('ricecoder.chat'), 'Should find chat command');
			assert.ok(getCommandById('ricecoder.review'), 'Should find review command');
			assert.ok(getCommandById('ricecoder.generate'), 'Should find generate command');
			assert.ok(getCommandById('ricecoder.refactor'), 'Should find refactor command');
		});
	});

	describe('getKeyboardShortcut', () => {
		it('should return keyboard shortcut for command', () => {
			const shortcut = getKeyboardShortcut('ricecoder.chat');
			assert.ok(shortcut, 'Should return keyboard shortcut');
			assert.ok(
				shortcut === 'ctrl+shift+r' || shortcut === 'cmd+shift+r',
				'Should return valid shortcut'
			);
		});

		it('should return undefined for unknown command', () => {
			const shortcut = getKeyboardShortcut('ricecoder.unknown');
			assert.strictEqual(shortcut, undefined, 'Should return undefined for unknown command');
		});

		it('should return different shortcuts for different commands', () => {
			const chatShortcut = getKeyboardShortcut('ricecoder.chat');
			const reviewShortcut = getKeyboardShortcut('ricecoder.review');
			assert.notStrictEqual(chatShortcut, reviewShortcut, 'Different commands should have different shortcuts');
		});
	});

	describe('Command Uniqueness', () => {
		it('should have unique command IDs', () => {
			const commands = getAvailableCommands();
			const ids = commands.map(cmd => cmd.id);
			const uniqueIds = new Set(ids);
			assert.strictEqual(ids.length, uniqueIds.size, 'All command IDs should be unique');
		});

		it('should have unique Windows keybindings', () => {
			const commands = getAvailableCommands();
			const shortcuts = commands.map(cmd => cmd.keybinding.windows);
			const uniqueShortcuts = new Set(shortcuts);
			assert.strictEqual(shortcuts.length, uniqueShortcuts.size, 'All Windows shortcuts should be unique');
		});

		it('should have unique Mac keybindings', () => {
			const commands = getAvailableCommands();
			const shortcuts = commands.map(cmd => cmd.keybinding.mac);
			const uniqueShortcuts = new Set(shortcuts);
			assert.strictEqual(shortcuts.length, uniqueShortcuts.size, 'All Mac shortcuts should be unique');
		});
	});

	describe('Command Metadata Validation', () => {
		it('should have non-empty titles', () => {
			const commands = getAvailableCommands();
			commands.forEach(cmd => {
				assert.ok(cmd.title.length > 0, `Command ${cmd.id} should have non-empty title`);
			});
		});

		it('should have non-empty descriptions', () => {
			const commands = getAvailableCommands();
			commands.forEach(cmd => {
				assert.ok(cmd.description.length > 0, `Command ${cmd.id} should have non-empty description`);
			});
		});

		it('should have valid keybinding format', () => {
			const commands = getAvailableCommands();
			commands.forEach(cmd => {
				assert.ok(
					cmd.keybinding.windows.includes('+') || cmd.keybinding.windows.includes('-'),
					`Command ${cmd.id} should have valid Windows keybinding format`
				);
				assert.ok(
					cmd.keybinding.mac.includes('+') || cmd.keybinding.mac.includes('-'),
					`Command ${cmd.id} should have valid Mac keybinding format`
				);
			});
		});
	});
});
