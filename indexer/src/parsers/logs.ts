import { PublicKey } from '@solana/web3.js';
import * as crypto from 'crypto';

export interface ParsedEvent {
  name: string;
  data: any;
}

export function parseTransactionLogs(logs: string[]): ParsedEvent[] {
  const events: ParsedEvent[] = [];
  
  for (const log of logs) {
    if (log.startsWith('Program data: ')) {
      try {
        const b64Data = log.replace('Program data: ', '').trim();
        const rawBytes = Buffer.from(b64Data, 'base64');
        
        const discriminator = rawBytes.slice(0, 8);
        const data = rawBytes.slice(8);
        
        const orderCommittedDisc = crypto.createHash('sha256').update('event:OrderCommitted').digest().slice(0, 8);
        const swapExecutedDisc = crypto.createHash('sha256').update('event:SwapExecuted').digest().slice(0, 8);
        
        if (discriminator.equals(orderCommittedDisc)) {
          events.push({
            name: 'OrderCommitted',
            data: { rawHex: data.toString('hex') } 
          });
        } else if (discriminator.equals(swapExecutedDisc)) {
          events.push({
            name: 'SwapExecuted',
            data: { rawHex: data.toString('hex') }
          });
        }
      } catch (e) {
        console.error('Failed to parse log:', e);
      }
    }
  }
  
  return events;
}
