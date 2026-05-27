import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  NATIVE_MINT,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

const BASTION_AMM_PROGRAM = new PublicKey("BvFgtfCEeCcMHoN1PRHSXkdzVYTka1NsrVBTeHmnDN2D");

async function main() {
  console.log("═══════════════════════════════════════════");
  console.log("  Bastion Protocol — Full Pool Initialization");
  console.log("═══════════════════════════════════════════");

  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  const poolProgram: any = anchor.workspace.BastionPool;
  const ammProgram: any = anchor.workspace.BastionAmm;

  // ─── Step 1: Create USDC mock token mint ───
  console.log("\n[1/6] Creating USDC mock mint...");
  const usdcMint = await createMint(
    connection,
    wallet.payer,
    wallet.publicKey,
    null,
    6 // USDC has 6 decimals
  );
  console.log(`  USDC Mint: ${usdcMint.toBase58()}`);

  // ─── Step 2: Initialize BastionPool config ───
  console.log("\n[2/6] Initializing Bastion Pool config...");
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    poolProgram.programId
  );

  try {
    const tx1 = await poolProgram.methods
      .initialize(
        wallet.publicKey,   // treasury
        usdcMint,           // token_mint (USDC)
        NATIVE_MINT,        // wsol_mint
        BASTION_AMM_PROGRAM // amm_program
      )
      .accounts({
        config: configPda,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log(`  Config PDA: ${configPda.toBase58()}`);
    console.log(`  TX: ${tx1}`);
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  Config already initialized, skipping.");
    } else {
      throw e;
    }
  }

  // ─── Step 3: Create deposit vaults via program ───
  console.log("\n[3/6] Creating deposit vaults (WSOL + USDC)...");
  const [wsolVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("wsol_vault"), configPda.toBuffer()],
    poolProgram.programId
  );
  const [tokenVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("token_vault"), configPda.toBuffer()],
    poolProgram.programId
  );

  try {
    const tx3 = await poolProgram.methods
      .createVaults()
      .accounts({
        config: configPda,
        authority: wallet.publicKey,
        wsolVault: wsolVault,
        tokenVault: tokenVault,
        wsolMint: NATIVE_MINT,
        tokenMint: usdcMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    console.log(`  WSOL Vault:  ${wsolVault.toBase58()}`);
    console.log(`  Token Vault: ${tokenVault.toBase58()}`);
    console.log(`  TX: ${tx3}`);
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  Vaults already created, skipping.");
    } else {
      console.log(`  Vault creation error: ${e.message}`);
      if (e.logs) {
        const relevantLogs = e.logs.filter((l: string) => l.includes("Error") || l.includes("failed"));
        relevantLogs.forEach((l: string) => console.log(`    ${l}`));
      }
    }
  }

  // ─── Step 4: Initialize AMM Pool ───
  console.log("\n[4/6] Initializing AMM Pool (SOL/USDC)...");

  // Mint A = WSOL (NATIVE_MINT), Mint B = USDC
  const mintA = NATIVE_MINT;
  const mintB = usdcMint;

  const [ammPoolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), mintA.toBuffer(), mintB.toBuffer()],
    ammProgram.programId
  );

  // All vaults and LP mint are PDAs derived by the program
  const [vaultA] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_a"), ammPoolPda.toBuffer()],
    ammProgram.programId
  );
  const [vaultB] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_b"), ammPoolPda.toBuffer()],
    ammProgram.programId
  );
  const [lpMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("lp_mint"), ammPoolPda.toBuffer()],
    ammProgram.programId
  );
  const [poolLpVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("lp_vault"), ammPoolPda.toBuffer()],
    ammProgram.programId
  );

  try {
    const tx2 = await ammProgram.methods
      .initializePool(30) // 30 bps = 0.3% fee
      .accounts({
        pool: ammPoolPda,
        mintA: mintA,
        mintB: mintB,
        vaultA: vaultA,
        vaultB: vaultB,
        lpMint: lpMintPda,
        poolLpVault: poolLpVault,
        authority: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    console.log(`  AMM Pool PDA: ${ammPoolPda.toBase58()}`);
    console.log(`  Vault A (WSOL): ${vaultA.toBase58()}`);
    console.log(`  Vault B (USDC): ${vaultB.toBase58()}`);
    console.log(`  LP Mint: ${lpMintPda.toBase58()}`);
    console.log(`  TX: ${tx2}`);
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  AMM Pool already initialized, skipping.");
    } else {
      console.log(`  AMM init error: ${e.message}`);
      if (e.logs) {
        const relevantLogs = e.logs.filter((l: string) => l.includes("Error") || l.includes("failed"));
        relevantLogs.forEach((l: string) => console.log(`    ${l}`));
      }
    }
  }

  // ─── Step 5: Mint test USDC to wallet ───
  console.log("\n[5/6] Minting test USDC to wallet...");
  const userUsdcAta = await getOrCreateAssociatedTokenAccount(
    connection,
    wallet.payer,
    usdcMint,
    wallet.publicKey
  );

  await mintTo(
    connection,
    wallet.payer,
    usdcMint,
    userUsdcAta.address,
    wallet.payer,
    10_000_000_000 // 10,000 USDC
  );
  console.log(`  Minted 10,000 USDC to ${userUsdcAta.address.toBase58()}`);

  // ─── Step 6: Summary ───
  console.log("\n═══════════════════════════════════════════");
  console.log("  Initialization Complete!");
  console.log("═══════════════════════════════════════════");
  console.log(`  Pool Program:  ${poolProgram.programId.toBase58()}`);
  console.log(`  AMM Program:   ${ammProgram.programId.toBase58()}`);
  console.log(`  Config PDA:    ${configPda.toBase58()}`);
  console.log(`  USDC Mint:     ${usdcMint.toBase58()}`);
  console.log(`  WSOL Mint:     ${NATIVE_MINT.toBase58()}`);
  console.log("");
  console.log("  Run 'bastion' to start trading!");
  console.log("═══════════════════════════════════════════");
}

main().catch((e) => {
  console.error(`\nError: ${e.message || e}`);
  if (e.logs) {
    console.error("\nProgram logs:");
    e.logs.forEach((l: string) => console.error(`  ${l}`));
  }
  process.exit(1);
});
