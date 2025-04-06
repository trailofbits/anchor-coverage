import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { Basic } from "../target/types/basic";
import * as assert from "assert";

describe("basic", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Basic as Program<Basic>;

  const [storage_address] = PublicKey.findProgramAddressSync(
    [Buffer.from("")],
    program.programId,
  );

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Should increment `y`", async () => {
    const before = await program.account.storage.fetch(storage_address);
    await program.methods.incrementY().rpc();
    const after = await program.account.storage.fetch(storage_address);
    assert.ok(before.x.eq(after.x));
    assert.ok(before.y.addn(1).eq(after.y));
  });
});
