import { Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

async function main() {
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    const walletPath = path.join(os.homedir(), '.config', 'solana', 'id.json');
    const secretKey = JSON.parse(fs.readFileSync(walletPath, 'utf-8'));
    const pubkey = Keypair.fromSecretKey(Uint8Array.from(secretKey)).publicKey;
    
    console.log(`Requesting airdrop for ${pubkey.toBase58()}...`);
    const sig = await connection.requestAirdrop(pubkey, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig);
    
    console.log("Airdrop successful.");
}

main().catch(console.error);
