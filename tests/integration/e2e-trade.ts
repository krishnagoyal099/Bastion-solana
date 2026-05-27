import { BastionClient } from "@bastion/sdk";
import { Keypair, PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("E2E Trade Flow", () => {
  it("submits a trade through the SDK to the relayer", async () => {
    const client = new BastionClient({
      rpcUrl: "http://127.0.0.1:8899",
      relayerUrl: "http://127.0.0.1:3000",
      relayerPubKey: Buffer.alloc(32, 1)
    });

    const wallet = Keypair.generate();
    const dummyMint = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

    try {
      const response = await client.darkPool.submitOrder({
        beneficiary: wallet.publicKey,
        side: "buy",
        amount: 1000000000, 
        price: 150000000,   
        tokenMint: dummyMint
      });

      assert.isNotNull(response.commitment);
      assert.equal(response.result.status, "success");
    } catch (e: any) {
      if (e.message.includes("ECONNREFUSED")) {
        console.warn("Relayer not running, skipping test");
      } else {
        throw e;
      }
    }
  });
});
