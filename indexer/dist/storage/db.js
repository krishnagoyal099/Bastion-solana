"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.EventStorage = void 0;
const sqlite3_1 = __importDefault(require("sqlite3"));
const util_1 = require("util");
class EventStorage {
    constructor(dbPath = ':memory:') {
        this.db = new sqlite3_1.default.Database(dbPath);
    }
    async init() {
        const run = (0, util_1.promisify)(this.db.run.bind(this.db));
        await run(`
      CREATE TABLE IF NOT EXISTS events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        signature TEXT NOT NULL,
        program_id TEXT NOT NULL,
        event_name TEXT NOT NULL,
        event_data TEXT NOT NULL,
        slot INTEGER NOT NULL,
        timestamp INTEGER NOT NULL
      )
    `);
        await run(`CREATE INDEX IF NOT EXISTS idx_events_name ON events(event_name)`);
    }
    async insertEvent(signature, programId, eventName, eventData, slot, timestamp) {
        return new Promise((resolve, reject) => {
            this.db.run(`INSERT INTO events (signature, program_id, event_name, event_data, slot, timestamp) VALUES (?, ?, ?, ?, ?, ?)`, [signature, programId, eventName, JSON.stringify(eventData), slot, timestamp], function (err) {
                if (err)
                    reject(err);
                else
                    resolve();
            });
        });
    }
    async getEventsByType(eventName, limit = 50) {
        return new Promise((resolve, reject) => {
            this.db.all(`SELECT * FROM events WHERE event_name = ? ORDER BY timestamp DESC LIMIT ?`, [eventName, limit], (err, rows) => {
                if (err)
                    reject(err);
                else
                    resolve(rows.map(r => ({
                        ...r,
                        event_data: JSON.parse(r.event_data)
                    })));
            });
        });
    }
}
exports.EventStorage = EventStorage;
