// KairosOS Web Dashboard v2 — Node.js backend with MCP integration
const http = require('http');
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const net = require('net');

const PORT = process.env.PORT || 8080;
const STATIC_DIR = path.join(__dirname, 'public');
const MCP_SOCKET = process.env.MCP_SOCKET || '/run/kairos/mcp.sock';
const BPF_SOCKET = process.env.BPF_SOCKET || '/run/kairos/bpf.sock';
const PKG_SOCKET = process.env.PKG_SOCKET || '/run/kairos/pkg.sock';
const CONF_FILE = process.env.CONF_FILE || '/etc/kairos/configuration.nix';
const GENERATIONS_DIR = process.env.GENERATIONS_DIR || '/var/lib/kairos/generations';

const CLIENTS = [];

const MIME_TYPES = {
  '.html': 'text/html', '.css': 'text/css', '.js': 'application/javascript',
  '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
};

function serveStatic(res, urlPath) {
  const filePath = path.join(STATIC_DIR, urlPath === '/' ? 'index.html' : urlPath);
  if (!filePath.startsWith(STATIC_DIR)) { res.writeHead(403); res.end('Forbidden'); return; }
  const ext = path.extname(filePath);
  fs.readFile(filePath, (err, data) => {
    if (err) { res.writeHead(404); res.end('Not found'); return; }
    res.writeHead(200, { 'Content-Type': MIME_TYPES[ext] || 'application/octet-stream' });
    res.end(data);
  });
}

function sendSSE(data) {
  const msg = `data: ${JSON.stringify(data)}\n\n`;
  CLIENTS.forEach(c => { try { c.write(msg); } catch { /* drop */ } });
}

function mcpRequest(socketPath, method, params) {
  return new Promise((resolve, reject) => {
    const client = net.createConnection(socketPath, () => {
      const req = JSON.stringify({ jsonrpc: '2.0', id: 1, method, params: params || {} });
      client.write(req + '\n');
    });
    let buf = '';
    client.on('data', d => { buf += d.toString(); });
    client.on('end', () => {
      try { resolve(JSON.parse(buf)); } catch { reject(new Error('Invalid MCP response')); }
    });
    client.on('error', reject);
    setTimeout(() => { client.destroy(); reject(new Error('MCP timeout')); }, 5000);
  });
}

async function handleApi(req, res) {
  const url = new URL(req.url, `http://localhost${PORT}`);
  const method = req.method;
  const respond = (code, data) => {
    res.writeHead(code, {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
    });
    res.end(JSON.stringify(data));
  };

  try {
    // System status
    if (method === 'GET' && url.pathname === '/api/status') {
      const status = {
        version: '0.2.0',
        hostname: require('os').hostname(),
        uptime: process.uptime(),
        services: {},
        telemetry: null,
      };
      for (const svc of ['kairos-bpf', 'kairos-mcp', 'kairos-git-logger', 'kairos-apply', 'kairos-pkg', 'kairos-hermes']) {
        try {
          const s = spawn('systemctl', ['is-active', svc]);
          status.services[svc] = await new Promise(r => {
            let o = ''; s.stdout.on('data', d => o += d);
            s.on('close', () => r(o.trim()));
          });
        } catch { status.services[svc] = 'unknown'; }
      }
      respond(200, status);
      return;
    }

    // eBPF telemetry
    if (method === 'GET' && url.pathname.startsWith('/api/bpf/')) {
      const endpoint = url.pathname.replace('/api/bpf/', '');
      try {
        const result = await mcpRequest(BPF_SOCKET, 'resources/read', {
          uri: `kairos://bpf/${endpoint}${url.search}`
        });
        respond(200, result);
      } catch (e) {
        respond(200, { error: e.message, endpoint, fallback: true, data: await getBpfFallback(endpoint) });
      }
      return;
    }

    // Knowledge graph query
    if (method === 'POST' && url.pathname === '/api/kg/query') {
      let body = '';
      await new Promise(r => { req.on('data', c => body += c); req.on('end', r); });
      const { q, top_k = 10 } = JSON.parse(body);
      try {
        const result = await mcpRequest(PKG_SOCKET, 'tools/call', {
          name: 'query', arguments: { q, top_k }
        });
        respond(200, result);
      } catch {
        respond(200, { results: [], error: 'PKG unavailable' });
      }
      return;
    }

    // Knowledge graph stats
    if (method === 'GET' && url.pathname === '/api/kg/stats') {
      try {
        const result = await mcpRequest(PKG_SOCKET, 'tools/call', { name: 'stats', arguments: {} });
        respond(200, result);
      } catch {
        respond(200, { entities: 0, relationships: 0, error: 'PKG unavailable' });
      }
      return;
    }

    // Config file read/write
    if (method === 'GET' && url.pathname === '/api/config') {
      try {
        const content = fs.readFileSync(CONF_FILE, 'utf-8');
        respond(200, { content, path: CONF_FILE });
      } catch (e) {
        respond(200, { content: '', error: e.message });
      }
      return;
    }
    if (method === 'POST' && url.pathname === '/api/config') {
      let body = '';
      await new Promise(r => { req.on('data', c => body += c); req.on('end', r); });
      try {
        const { content } = JSON.parse(body);
        const tmp = CONF_FILE + '.tmp';
        fs.writeFileSync(tmp, content, 'utf-8');
        fs.renameSync(tmp, CONF_FILE);
        respond(200, { ok: true });
      } catch (e) {
        respond(500, { ok: false, error: e.message });
      }
      return;
    }

    // Generations list
    if (method === 'GET' && url.pathname === '/api/generations') {
      try {
        const gens = fs.readdirSync(GENERATIONS_DIR).filter(f => f.startsWith('gen-'));
        const genMeta = gens.map(g => {
          try {
            const meta = JSON.parse(fs.readFileSync(path.join(GENERATIONS_DIR, g, 'meta.json'), 'utf-8'));
            return { id: g, ...meta };
          } catch { return { id: g }; }
        });
        respond(200, genMeta.sort((a, b) => (b.timestamp || 0) - (a.timestamp || 0)));
      } catch {
        respond(200, []);
      }
      return;
    }

    // Chat
    if (method === 'POST' && url.pathname === '/api/chat') {
      let body = '';
      await new Promise(r => { req.on('data', c => body += c); req.on('end', r); });
      const { message } = JSON.parse(body);
      const hermes = spawn('hermes', ['--json', '--eval', message], {
        timeout: 30000, cwd: '/tmp',
      });
      let out = '';
      hermes.stdout.on('data', d => out += d);
      hermes.stderr.on('data', d => out += d);
      hermes.on('close', code => respond(200, { response: out, code }));
      return;
    }

    // Logs
    if (method === 'GET' && url.pathname === '/api/logs') {
      const lines = parseInt(url.searchParams.get('lines')) || 50;
      const svc = url.searchParams.get('service') || 'kairos-hermes';
      const journal = spawn('journalctl', ['-u', svc, '-n', String(lines), '--no-pager', '-o', 'short-iso']);
      let out = '';
      journal.stdout.on('data', d => out += d);
      journal.on('close', () => respond(200, { logs: out }));
      return;
    }

    // SSE for live telemetry
    if (method === 'GET' && url.pathname === '/api/events') {
      res.writeHead(200, {
        'Content-Type': 'text/event-stream',
        'Cache-Control': 'no-cache',
        Connection: 'keep-alive',
        'Access-Control-Allow-Origin': '*',
      });
      CLIENTS.push(res);
      req.on('close', () => {
        const i = CLIENTS.indexOf(res);
        if (i >= 0) CLIENTS.splice(i, 1);
      });
      return;
    }

    respond(404, { error: 'Not found' });
  } catch (e) {
    respond(500, { error: e.message });
  }
}

function getBpfFallback(endpoint) {
  try {
    const stat = require('os');
    switch (endpoint) {
      case 'stats': return {
        active_probes: 0, total_events: 0, dropped_events: 0,
        memory_kb: 0, uptime_seconds: process.uptime(),
        info: 'eBPF unavailable — showing system metrics',
        cpu_load: stat.loadavg(),
        freemem: stat.freemem(),
        totalmem: stat.totalmem(),
      };
      default: return { info: 'eBPF unavailable in this environment' };
    }
  } catch { return { info: 'eBPF unavailable' }; }
}

// Periodic telemetry broadcast
setInterval(async () => {
  try {
    const result = await mcpRequest(BPF_SOCKET, 'resources/read', { uri: 'kairos://bpf/stats' });
    sendSSE({ type: 'bpf', data: result });
  } catch {
    sendSSE({ type: 'bpf', data: getBpfFallback('stats') });
  }
}, 5000);

const server = http.createServer((req, res) => {
  if (req.url.startsWith('/api/')) handleApi(req, res);
  else serveStatic(res, req.url);
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`KairosOS Web Dashboard v2 running on http://0.0.0.0:${PORT}`);
  console.log(`  MCP: ${MCP_SOCKET}`);
  console.log(`  BPF: ${BPF_SOCKET}`);
  console.log(`  PKG: ${PKG_SOCKET}`);
});
