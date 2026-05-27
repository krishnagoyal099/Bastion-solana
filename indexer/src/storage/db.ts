import sqlite3 from 'sqlite3';
import { promisify } from 'util';

export class EventStorage {
  private db: sqlite3.Database;

  constructor(dbPath: string = ':memory:') {
    this.db = new sqlite3.Database(dbPath);
  }

  async init() {
    const run = promisify(this.db.run.bind(this.db));
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

  async insertEvent(signature: string, programId: string, eventName: string, eventData: any, slot: number, timestamp: number): Promise<void> {
    return new Promise((resolve, reject) => {
      this.db.run(
        `INSERT INTO events (signature, program_id, event_name, event_data, slot, timestamp) VALUES (?, ?, ?, ?, ?, ?)`,
        [signature, programId, eventName, JSON.stringify(eventData), slot, timestamp],
        function(err) {
          if (err) reject(err);
          else resolve();
        }
      );
    });
  }

  async getEventsByType(eventName: string, limit: number = 50): Promise<any[]> {
    return new Promise((resolve, reject) => {
      this.db.all(
        `SELECT * FROM events WHERE event_name = ? ORDER BY timestamp DESC LIMIT ?`,
        [eventName, limit],
        (err, rows: any[]) => {
          if (err) reject(err);
          else resolve(rows.map(r => ({
            ...r,
            event_data: JSON.parse(r.event_data)
          })));
        }
      );
    });
  }
}
