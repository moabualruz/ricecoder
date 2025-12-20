#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

// Start the MCP server
console.log('Starting MCP server...');
const server = spawn('cargo', ['run', '--package', 'ricegrep', '--', 'mcp'], {
    cwd: __dirname,
    stdio: ['pipe', 'pipe', 'inherit']
});

let messageId = 1;
let responses = [];

// Send initialize message
function sendInitialize() {
    const message = {
        jsonrpc: "2.0",
        id: messageId++,
        method: "initialize",
        params: {
            protocolVersion: "2024-11-05",
            capabilities: {},
            clientInfo: {
                name: "test-client",
                version: "1.0.0"
            }
        }
    };

    console.log('Sending initialize message:', JSON.stringify(message, null, 2));
    server.stdin.write(JSON.stringify(message) + '\n');
}

// Send tools/list message
function sendToolsList() {
    const message = {
        jsonrpc: "2.0",
        id: messageId++,
        method: "tools/list"
    };

    console.log('Sending tools/list message:', JSON.stringify(message, null, 2));
    server.stdin.write(JSON.stringify(message) + '\n');
}

// Send tools/call message
function sendToolsCall() {
    const message = {
        jsonrpc: "2.0",
        id: messageId++,
        method: "tools/call",
        params: {
            name: "search",
            arguments: {
                pattern: "fn main",
                path: ".",
                ai_enhanced: true
            }
        }
    };

    console.log('Sending tools/call message:', JSON.stringify(message, null, 2));
    server.stdin.write(JSON.stringify(message) + '\n');
}

// Handle server responses
server.stdout.on('data', (data) => {
    const response = data.toString().trim();
    console.log('Server response:', response);
    responses.push(response);

    try {
        const parsed = JSON.parse(response);
        console.log('Parsed response:', JSON.stringify(parsed, null, 2));
    } catch (e) {
        console.log('Failed to parse response as JSON');
    }
});

server.on('close', (code) => {
    console.log(`Server exited with code ${code}`);
    console.log('Total responses received:', responses.length);
    process.exit(0);
});

// Send messages with delays
setTimeout(sendInitialize, 2000);
setTimeout(sendToolsList, 4000);
setTimeout(sendToolsCall, 6000);

// Exit after 10 seconds
setTimeout(() => {
    console.log('Timeout reached, killing server...');
    server.kill();
}, 10000);