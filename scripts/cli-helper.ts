/**
 * Bastion CLI Helper — Invokes Anchor program instructions from the CLI
 *
 * Usage:
 *   npx ts-node scripts/cli-helper.ts \
 *     --program pool --method deposit_sol --amount 1000000000 \
 *     --keypair ./keys/user.json --rpc http://127.0.0.1:8899
 */

import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  NATIVE_MINT,
} from "@solana/spl-token";
import * as fs from "fs";

// Parse CLI arguments
function parseArgs(): Record<string, string> {
  const args: Record<string, string> = {};
  const argv = process.argv.slice(2);
  for (let i = 0; i < argv.length; i++) {
    if (argv[i].startsWith("--")) {
      const key = argv[i].substring(2);
      const value = argv[i + 1] && !argv[i + 1].startsWith("--") ? argv[i + 1] : "true";
      args[key] = value;
      if (value !== "true") i++;
    }
  }
  return args;
}

async function main() {
  const args = parseArgs();

  const rpcUrl = args.rpc || "http://127.0.0.1:8899";
  const keypairPath = args.keypair;
  const programName = args.program; // "pool" or "amm"
  const method = args.method;

  if (!keypairPath || !programName || !method) {
    console.error("ERROR:Missing required args: --program, --method, --keypair");
    process.exit(1);
  }

  // Load keypair
  let keypair: Keypair;
  try {
    const raw = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
    keypair = Keypair.fromSecretKey(Uint8Array.from(raw));
  } catch (e: any) {
    console.error(`ERROR:Failed to load keypair: ${e.message}`);
    process.exit(1);
  }

  // Create connection & provider
  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new Wallet(keypair);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  anchor.setProvider(provider);

  // Load IDL
  const POOL_PROGRAM_ID = new PublicKey("CbHS6twCMkYyodaEUtvonRV6HVBZnkGjekohLqXJziU5");
  const AMM_PROGRAM_ID = new PublicKey("BvFgtfCEeCcMHoN1PRHSXkdzVYTka1NsrVBTeHmnDN2D");

  try {
    if (programName === "pool") {
      await handlePoolMethod(provider, connection, keypair, method, args, POOL_PROGRAM_ID);
    } else if (programName === "amm") {
      await handleAmmMethod(provider, connection, keypair, method, args, AMM_PROGRAM_ID);
    } else {
      console.error(`ERROR:Unknown program: ${programName}`);
      process.exit(1);
    }
  } catch (e: any) {
    console.error(`ERROR:${e.message || e}`);
    process.exit(1);
  }
}

// ═══════════════════════════════════════════════════════════════════
// Pool Program Methods
// ═══════════════════════════════════════════════════════════════════

async function handlePoolMethod(
  provider: AnchorProvider,
  connection: Connection,
  keypair: Keypair,
  method: string,
  args: Record<string, string>,
  programId: PublicKey
) {
  // Load IDL from file
  const idlPath = `${__dirname}/../target/idl/bastion_pool.json`;
  if (!fs.existsSync(idlPath)) {
    console.error("ERROR:IDL not found. Run 'anchor build' first.");
    process.exit(1);
  }
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
  const program: any = new Program(idl, provider);

  const user = keypair.publicKey;

  // Derive common PDAs
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    programId
  );
  const [userDepositPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("deposit"), user.toBuffer()],
    programId
  );

  switch (method) {
    case "deposit_sol": {
      const amount = new anchor.BN(args.amount || "0");

      // Derive WSOL vault PDA
      const [wsolVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("wsol_vault"), configPda.toBuffer()],
        programId
      );

      const tx = await program.methods
        .depositSol(amount)
        .accounts({
          userDeposit: userDepositPda,
          user: user,
          wsolVault: wsolVault,
          wsolMint: NATIVE_MINT,
          config: configPda,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([keypair])
        .rpc();

      console.log(`TX:${tx}`);
      break;
    }

    case "withdraw": {
      const amount = new anchor.BN(args.amount || "0");
      const isSol = args["is-sol"] !== "false";

      // Derive the correct program vault based on is_sol
      const vaultSeed = isSol ? "wsol_vault" : "token_vault";
      const [programVault] = PublicKey.findProgramAddressSync(
        [Buffer.from(vaultSeed), configPda.toBuffer()],
        programId
      );

      // Read config to get the token mint
      const configData = await connection.getAccountInfo(configPda);
      if (!configData) {
        console.error("ERROR:Config not initialized");
        process.exit(1);
      }
      // Config layout after 8-byte discriminator:
      // authority(32), amm_program(32), treasury(32), token_mint(32), wsol_mint(32)
      const configOffset = 8;
      const tokenMintFromConfig = new PublicKey(
        configData.data.subarray(configOffset + 96, configOffset + 128)
      );
      const wsolMintFromConfig = new PublicKey(
        configData.data.subarray(configOffset + 128, configOffset + 160)
      );

      // Get user's ATA for the relevant mint
      const withdrawMint = isSol ? wsolMintFromConfig : tokenMintFromConfig;

      // Ensure user ATA exists (create if needed)
      const userAtaAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        keypair, // payer
        withdrawMint,
        user
      );
      const userAta = userAtaAccount.address;

      const tx = await program.methods
        .withdraw(amount, isSol)
        .accounts({
          userDeposit: userDepositPda,
          user: user,
          programVault: programVault,
          userAta: userAta,
          config: configPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([keypair])
        .rpc();

      console.log(`TX:${tx}`);
      break;
    }

    case "deposit_token": {
      const amount = new anchor.BN(args.amount || "0");

      const [tokenVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("token_vault"), configPda.toBuffer()],
        programId
      );

      // Get user's token ATA
      const tokenMint = new PublicKey(args.mint || "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
      const userTokenAta = await getAssociatedTokenAddress(tokenMint, user);

      const tx = await program.methods
        .depositToken(amount)
        .accounts({
          userDeposit: userDepositPda,
          user: user,
          userTokenAccount: userTokenAta,
          programVault: tokenVault,
          tokenMint: tokenMint,
          config: configPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([keypair])
        .rpc();

      console.log(`TX:${tx}`);
      break;
    }

    default:
      console.error(`ERROR:Unknown pool method: ${method}`);
      process.exit(1);
  }
}

// ═══════════════════════════════════════════════════════════════════
// AMM Program Methods
// ═══════════════════════════════════════════════════════════════════

async function handleAmmMethod(
  provider: AnchorProvider,
  connection: Connection,
  keypair: Keypair,
  method: string,
  args: Record<string, string>,
  programId: PublicKey
) {
  const idlPath = `${__dirname}/../target/idl/bastion_amm.json`;
  if (!fs.existsSync(idlPath)) {
    console.error("ERROR:IDL not found. Run 'anchor build' first.");
    process.exit(1);
  }
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
  const program: any = new Program(idl, provider);

  const user = keypair.publicKey;

  // Get pool accounts — find the first pool
  const poolAccounts = await connection.getProgramAccounts(programId, {
    filters: [{ dataSize: 250 }], // AmmPool size (8 + 242)
  });

  if (poolAccounts.length === 0) {
    console.error("ERROR:No AMM pool found. Initialize pool first.");
    process.exit(1);
  }

  const poolPubkey = poolAccounts[0].pubkey;
  const poolData = poolAccounts[0].account.data;

  // Parse pool data to get vault addresses
  const offset = 8; // discriminator
  const mintA = new PublicKey(poolData.subarray(offset + 32, offset + 64));
  const mintB = new PublicKey(poolData.subarray(offset + 64, offset + 96));
  const vaultA = new PublicKey(poolData.subarray(offset + 96, offset + 128));
  const vaultB = new PublicKey(poolData.subarray(offset + 128, offset + 160));
  const lpMint = new PublicKey(poolData.subarray(offset + 160, offset + 192));

  switch (method) {
    case "add_liquidity": {
      const amountA = new anchor.BN(args["amount-a"] || "0");
      const amountB = new anchor.BN(args["amount-b"] || "0");
      const minLp = new anchor.BN(args["min-lp"] || "0");

      const [lpPosition] = PublicKey.findProgramAddressSync(
        [Buffer.from("lp"), poolPubkey.toBuffer(), user.toBuffer()],
        programId
      );
      const [poolLpVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("lp_vault"), poolPubkey.toBuffer()],
        programId
      );

      const providerTokenAAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        keypair,
        mintA,
        user
      );
      const providerTokenA = providerTokenAAccount.address;

      const providerTokenBAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        keypair,
        mintB,
        user
      );
      const providerTokenB = providerTokenBAccount.address;
      const providerLpAtaAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        keypair,
        lpMint,
        user
      );
      const providerLpAta = providerLpAtaAccount.address;

      const tx = await program.methods
        .addLiquidity(amountA, amountB, minLp)
        .accounts({
          pool: poolPubkey,
          lpPosition: lpPosition,
          vaultA: vaultA,
          vaultB: vaultB,
          lpMint: lpMint,
          poolLpVault: poolLpVault,
          providerLpAta: providerLpAta,
          providerTokenAAta: providerTokenA,
          providerTokenBAta: providerTokenB,
          provider: user,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([keypair])
        .rpc();

      console.log(`TX:${tx}`);
      break;
    }

    case "remove_liquidity": {
      const shares = new anchor.BN(args.shares || "0");
      const minA = new anchor.BN(args["min-a"] || "0");
      const minB = new anchor.BN(args["min-b"] || "0");

      const [lpPosition] = PublicKey.findProgramAddressSync(
        [Buffer.from("lp"), poolPubkey.toBuffer(), user.toBuffer()],
        programId
      );
      const [poolLpVault] = PublicKey.findProgramAddressSync(
        [Buffer.from("lp_vault"), poolPubkey.toBuffer()],
        programId
      );

      const providerTokenA = await getAssociatedTokenAddress(mintA, user);
      const providerTokenB = await getAssociatedTokenAddress(mintB, user);
      const providerLpAta = await getAssociatedTokenAddress(lpMint, user);

      const tx = await program.methods
        .removeLiquidity(shares, minA, minB)
        .accounts({
          pool: poolPubkey,
          lpPosition: lpPosition,
          vaultA: vaultA,
          vaultB: vaultB,
          lpMint: lpMint,
          poolLpVault: poolLpVault,
          providerLpAta: providerLpAta,
          providerTokenAAta: providerTokenA,
          providerTokenBAta: providerTokenB,
          provider: user,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([keypair])
        .rpc();

      console.log(`TX:${tx}`);
      break;
    }

    case "swap_a_to_b":
    case "swap_b_to_a": {
      const amountIn = new anchor.BN(args["amount-in"] || "0");
      const minOut = new anchor.BN(args["min-out"] || "0");

      const traderTokenA = await getAssociatedTokenAddress(mintA, user);
      const traderTokenB = await getAssociatedTokenAddress(mintB, user);

      const methodFn = method === "swap_a_to_b"
        ? program.methods.swapAToB(amountIn, minOut)
        : program.methods.swapBToA(amountIn, minOut);

      const tx = await methodFn
        .accounts({
          pool: poolPubkey,
          vaultA: vaultA,
          vaultB: vaultB,
          traderTokenAAta: traderTokenA,
          traderTokenBAta: traderTokenB,
          trader: user,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([keypair])
        .rpc();

      console.log(`TX:${tx}`);
      break;
    }

    default:
      console.error(`ERROR:Unknown AMM method: ${method}`);
      process.exit(1);
  }
}

main().catch((e) => {
  console.error(`ERROR:${e.message || e}`);
  process.exit(1);
});
