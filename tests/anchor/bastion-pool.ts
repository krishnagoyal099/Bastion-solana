import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";

describe("bastion-pool", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.BastionPool as Program<any>;
  const provider = anchor.getProvider();

  it("Initializes the dark pool", async () => {
    const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    try {
      await program.methods
        .initialize(new anchor.BN(5)) 
        .accounts({
          config: configPda,
          admin: provider.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
        
      const configState = await program.account.bastionConfig.fetch(configPda);
      assert.isTrue(configState.admin.equals(provider.publicKey));
      assert.equal(configState.protocolFee.toNumber(), 5);
    } catch (e: any) {
      if (!e.message.includes("already in use")) {
        throw e;
      }
    }
  });
});
