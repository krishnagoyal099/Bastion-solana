import express from 'express';
import cors from 'cors';
import { WebSocketServer, WebSocket } from 'ws';
import { createServer } from 'http';
import { EventStorage } from './storage/db';
import { parseTransactionLogs } from './parsers/logs';
import * as crypto from 'crypto';

const BASTION_PROGRAMS = [
  'BASTi0N11111111111111111111111111111111111111',
  'BASTAMM1111111111111111111111111111111111111'
];

// F16 FIX: Load webhook secret from environment
const WEBHOOK_SECRET = process.env.BASTION_WEBHOOK_SECRET || '';
if (!WEBHOOK_SECRET) {
  console.warn('WARNING: BASTION_WEBHOOK_SECRET not set. Webhook auth disabled (insecure).');
}

/**
 * F16 FIX: Verify webhook signature (HMAC-SHA256)
 * Compatible with Helius and similar webhook providers
 */
function verifyWebhookSignature(body: string, signature: string | undefined): boolean {
  if (!WEBHOOK_SECRET) return true; // Skip if no secret configured
  if (!signature) return false;

  const expectedSig = crypto
    .createHmac('sha256', WEBHOOK_SECRET)
    .update(body)
    .digest('hex');

  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expectedSig)
  );
}

async function main() {
  const app = express();

  // Store raw body for signature verification
  app.use(express.json({
    verify: (req: any, _res, buf) => {
      req.rawBody = buf.toString();
    }
  }));
  app.use(cors());

  const server = createServer(app);
  const wss = new WebSocketServer({ server, path: '/ws' });

  const storage = new EventStorage('./indexer.sqlite');
  await storage.init();

  const clients = new Set<WebSocket>();

  wss.on('connection', (ws) => {
    clients.add(ws);
    ws.on('close', () => clients.delete(ws));
  });

  app.post('/webhook', async (req: any, res) => {
    // F16 FIX: Verify webhook signature
    const signature = req.headers['x-webhook-signature'] as string | undefined;
    if (!verifyWebhookSignature(req.rawBody || '', signature)) {
      res.status(401).send('Unauthorized: invalid webhook signature');
      return;
    }

    const transactions = req.body;
    
    if (!Array.isArray(transactions)) {
      res.status(400).send('Bad Request: expected array of transactions');
      return;
    }

    // Rate limit: max 100 transactions per webhook call
    const txBatch = transactions.slice(0, 100);

    for (const tx of txBatch) {
      if (!tx || typeof tx.signature !== 'string') continue;

      const signature = tx.signature;
      const slot = typeof tx.slot === 'number' ? tx.slot : 0;
      const timestamp = typeof tx.timestamp === 'number' ? tx.timestamp : Math.floor(Date.now() / 1000);
      
      const logs: string[] = Array.isArray(tx.meta?.logMessages) ? tx.meta.logMessages : [];
      const events = parseTransactionLogs(logs);
      
      for (const event of events) {
        await storage.insertEvent(
          signature,
          'BastionProtocol',
          event.name,
          event.data,
          slot,
          timestamp
        );
        
        const wsPayload = JSON.stringify({ type: 'EVENT', data: event });
        for (const client of clients) {
          if (client.readyState === WebSocket.OPEN) {
            client.send(wsPayload);
          }
        }
      }
    }
    
    res.status(200).send('OK');
  });

  app.get('/events/:type', async (req, res) => {
    try {
      const eventType = req.params.type;
      // Sanitize: only allow alphanumeric event type names
      if (!/^[A-Za-z0-9]+$/.test(eventType)) {
        res.status(400).json({ success: false, error: 'Invalid event type' });
        return;
      }
      const events = await storage.getEventsByType(eventType);
      res.json({ success: true, events });
    } catch (e: any) {
      res.status(500).json({ success: false, error: 'Internal error' });
    }
  });

  app.get('/health', (_req, res) => {
    res.json({ status: 'ok', timestamp: Date.now() });
  });

  const PORT = process.env.PORT || 8080;
  server.listen(PORT, () => {
    console.log(`Bastion Indexer running on port ${PORT}`);
    console.log(`WebSocket Server running at ws://localhost:${PORT}/ws`);
  });
}

main().catch(console.error);
