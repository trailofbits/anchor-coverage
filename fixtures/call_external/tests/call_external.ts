import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CallExternal } from "../target/types/call_external";

describe("call_external", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.CallExternal as Program<CallExternal>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
    const txDetails = await program.provider.connection.getTransaction(tx, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    const logMessages = txDetails.meta.logMessages;
    console.log(`${JSON.stringify(logMessages, null, 2)}`);
  });
});
