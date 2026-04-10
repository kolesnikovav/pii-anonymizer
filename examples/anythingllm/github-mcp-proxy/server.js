const express = require('express');
const { spawn } = require('child_process');
const { v4: uuidv4 } = require('uuid');
const { createParser } = require('eventsource-parser');

const app = express();
app.use(express.json());

// Хранилище сессий
const sessions = new Map();

/**
 * Запускает MCP subprocess и обрабатывает JSON-RPC через stdio
 */
class MCPSubprocess {
  constructor(command, args, env) {
    this.command = command;
    this.args = args;
    this.env = env;
    this.child = null;
    this.messageId = 1;
    this.pendingRequests = new Map();
    this.buffer = '';
    this.initialized = false;
  }

  async start() {
    return new Promise((resolve, reject) => {
      console.log(`🔄 Запуск: ${this.command} ${this.args.join(' ')}`);

      this.child = spawn(this.command, this.args, {
        env: { ...process.env, ...this.env },
        stdio: ['pipe', 'pipe', 'inherit']
      });

      this.child.on('error', (err) => {
        console.error('❌ Subprocess error:', err.message);
        reject(err);
      });

      this.child.on('exit', (code) => {
        console.log(`⚠️ Subprocess exited with code ${code}`);
        this.initialized = false;
      });

      // Читаем stdout построчно (JSON-RPC messages)
      this.child.stdout.on('data', (data) => {
        const lines = data.toString().split('\n');
        for (const line of lines) {
          if (line.trim()) {
            this.handleResponse(line.trim());
          }
        }
      });

      // Отправляем initialize
      this.sendRequest('initialize', {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: { name: 'mcp-http-proxy', version: '0.1.0' }
      }).then((result) => {
        console.log('✅ MCP инициализирован');
        this.initialized = true;

        // Отправляем notification
        this.sendNotification('notifications/initialized');
        resolve();
      }).catch(reject);
    });
  }

  sendRequest(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = this.messageId++;
      const message = {
        jsonrpc: '2.0',
        id,
        method,
        params
      };

      this.pendingRequests.set(id, { resolve, reject, method });
      this.sendMessage(message);

      // Timeout
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error(`Timeout waiting for ${method}`));
        }
      }, 30000);
    });
  }

  sendNotification(method, params = {}) {
    this.sendMessage({
      jsonrpc: '2.0',
      method,
      params
    });
  }

  sendMessage(message) {
    if (!this.child || !this.child.stdin) {
      throw new Error('Subprocess not running');
    }
    this.child.stdin.write(JSON.stringify(message) + '\n');
  }

  handleResponse(line) {
    try {
      const response = JSON.parse(line);

      if (response.id !== undefined && this.pendingRequests.has(response.id)) {
        const { resolve, reject } = this.pendingRequests.get(response.id);
        this.pendingRequests.delete(response.id);

        if (response.error) {
          reject(new Error(response.error.message || 'MCP error'));
        } else {
          resolve(response.result);
        }
      }
    } catch (e) {
      console.warn('⚠️ Parse error:', e.message, line.substring(0, 100));
    }
  }

  async callTool(toolName, arguments_ = {}) {
    if (!this.initialized) {
      throw new Error('MCP not initialized');
    }

    return this.sendRequest('tools/call', {
      name: toolName,
      arguments: arguments_
    });
  }

  async listTools() {
    if (!this.initialized) {
      throw new Error('MCP not initialized');
    }

    const result = await this.sendRequest('tools/list');
    return result.tools || [];
  }

  stop() {
    if (this.child) {
      this.child.kill();
    }
  }
}

// SSE Endpoint
app.get('/sse', (req, res) => {
  const sessionId = uuidv4();
  const session = {
    id: sessionId,
    mcp: null,
    res
  };

  sessions.set(sessionId, session);

  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-cache',
    'Connection': 'keep-alive',
    'Access-Control-Allow-Origin': '*'
  });

  // Отправляем endpoint
  res.write(`event: endpoint\ndata: /message?sessionId=${sessionId}\nretry:3000000\n\n`);

  console.log(`📡 SSE сессия создана: ${sessionId}`);

  req.on('close', () => {
    sessions.delete(sessionId);
    console.log(`🔌 SSE сессия закрыта: ${sessionId}`);
  });
});

// Message endpoint
app.post('/message', async (req, res) => {
  const { sessionId } = req.query;
  const message = req.body;

  const session = sessions.get(sessionId);
  if (!session) {
    return res.status(404).json({ error: 'Session not found' });
  }

  try {
    // Лениво инициализируем MCP при первом запросе
    if (!session.mcp) {
      console.log('🔄 Инициализация GitHub MCP...');

      const mcp = new MCPSubprocess('docker', [
        'run', '-i', '--rm',
        '-e', `GITHUB_PERSONAL_ACCESS_TOKEN=${process.env.GITHUB_PERSONAL_ACCESS_TOKEN}`,
        '-e', `GITHUB_TOOLSETS=${process.env.GITHUB_TOOLSETS || 'all'}`,
        'ghcr.io/github/github-mcp-server'
      ], {});

      await mcp.start();
      session.mcp = mcp;
      console.log('✅ GitHub MCP готов');
    }

    // Обрабатываем JSON-RPC
    if (message.method === 'tools/list') {
      const tools = await session.mcp.listTools();
      const response = {
        jsonrpc: '2.0',
        id: message.id,
        result: { tools }
      };
      session.res.write(`data: ${JSON.stringify(response)}\n\n`);
    } else if (message.method === 'tools/call') {
      const result = await session.mcp.callTool(
        message.params.name,
        message.params.arguments || {}
      );
      const response = {
        jsonrpc: '2.0',
        id: message.id,
        result
      };
      session.res.write(`data: ${JSON.stringify(response)}\n\n`);
    } else if (message.method === 'initialize') {
      const response = {
        jsonrpc: '2.0',
        id: message.id,
        result: {
          protocolVersion: '2024-11-05',
          capabilities: { tools: { listChanged: true } },
          serverInfo: { name: 'GitHub MCP Server', version: 'latest' }
        }
      };
      session.res.write(`data: ${JSON.stringify(response)}\n\n`);
    } else if (message.method === 'ping') {
      const response = {
        jsonrpc: '2.0',
        id: message.id,
        result: {}
      };
      session.res.write(`data: ${JSON.stringify(response)}\n\n`);
    } else if (message.method === 'notifications/initialized') {
      // Просто ACK
      session.res.write(`data: ${JSON.stringify({ jsonrpc: '2.0', result: {} })}\n\n`);
    } else {
      const response = {
        jsonrpc: '2.0',
        id: message.id,
        error: { code: -32601, message: `Method not found: ${message.method}` }
      };
      session.res.write(`data: ${JSON.stringify(response)}\n\n`);
    }

    res.json({});
  } catch (error) {
    console.error('❌ Ошибка:', error.message);
    res.status(500).json({ error: error.message });
  }
});

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok', sessions: sessions.size });
});

const PORT = process.env.PORT || 3001;
app.listen(PORT, () => {
  console.log(`🌐 MCP HTTP Proxy запущен на порту ${PORT}`);
  console.log(`   SSE: http://localhost:${PORT}/sse`);
  console.log(`   Message: http://localhost:${PORT}/message`);
});
