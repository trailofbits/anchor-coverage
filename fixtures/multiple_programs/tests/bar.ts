import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { retry } from "retry";
import { Bar } from "../target/types/bar";
import * as assert from "assert";

describe("bar", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.bar as Program<Bar>;

  const [storage_address] = PublicKey.findProgramAddressSync(
    [Buffer.from("")],
    program.programId,
  );

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await retry(() => program.methods.initialize().rpc());
    console.log("Your transaction signature", tx);
  });

  it("Should increment `y`", async () => {
    const before = await program.account.storage.fetch(storage_address);
    await program.methods.incrementY().rpc();
    const after = await program.account.storage.fetch(storage_address);
    assert.ok(before.y.addn(1).eq(after.y));
  });
});
