"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const ws_1 = require("ws");
const http_1 = require("http");
const db_1 = require("./storage/db");
const logs_1 = require("./parsers/logs");
const BASTION_PROGRAMS = [
    'BASTi0N11111111111111111111111111111111111111',
    'BASTAMM1111111111111111111111111111111111111'
];
async function main() {
    const app = (0, express_1.default)();
    app.use(express_1.default.json());
    app.use((0, cors_1.default)());
    const server = (0, http_1.createServer)(app);
    const wss = new ws_1.WebSocketServer({ server, path: '/ws' });
    const storage = new db_1.EventStorage('./indexer.sqlite');
    await storage.init();
    const clients = new Set();
    wss.on('connection', (ws) => {
        clients.add(ws);
        ws.on('close', () => clients.delete(ws));
    });
    app.post('/webhook', async (req, res) => {
        const transactions = req.body;
        if (Array.isArray(transactions)) {
            for (const tx of transactions) {
                const signature = tx.signature;
                const slot = tx.slot || 0;
                const timestamp = tx.timestamp || Math.floor(Date.now() / 1000);
                const logs = tx.meta?.logMessages || [];
                const events = (0, logs_1.parseTransactionLogs)(logs);
                for (const event of events) {
                    await storage.insertEvent(signature, 'BastionProtocol', event.name, event.data, slot, timestamp);
                    const wsPayload = JSON.stringify({ type: 'EVENT', data: event });
                    for (const client of clients) {
                        if (client.readyState === ws_1.WebSocket.OPEN) {
                            client.send(wsPayload);
                        }
                    }
                }
            }
        }
        res.status(200).send('OK');
    });
    app.get('/events/:type', async (req, res) => {
        try {
            const type = req.params.type;
            const events = await storage.getEventsByType(type);
            res.json({ success: true, events });
        }
        catch (e) {
            res.status(500).json({ success: false, error: e.message });
        }
    });
    const PORT = process.env.PORT || 8080;
    server.listen(PORT, () => {
        console.log(`Bastion Indexer running on port ${PORT}`);
        console.log(`WebSocket Server running at ws://localhost:${PORT}/ws`);
    });
}
main().catch(console.error);
