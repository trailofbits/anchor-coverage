import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { MultipleTestConfigs } from "../target/types/multiple_test_configs";
import * as assert from "assert";

describe("multiple_test_configs", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.MultipleTestConfigs as Program<MultipleTestConfigs>;

  const [storage_address] = PublicKey.findProgramAddressSync(
    [Buffer.from("")],
    program.programId,
  );

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Should increment `x`", async () => {
    const before = await program.account.storage.fetch(storage_address);
    await program.methods.incrementX().rpc();
    const after = await program.account.storage.fetch(storage_address);
    assert.ok(before.x.addn(1).eq(after.x));
    assert.ok(before.y.eq(after.y));
  });

  it("Should increment `y`", async () => {
    const before = await program.account.storage.fetch(storage_address);
    await program.methods.incrementY().rpc();
    const after = await program.account.storage.fetch(storage_address);
    assert.ok(before.x.eq(after.x));
    assert.ok(before.y.addn(1).eq(after.y));
  });
});
