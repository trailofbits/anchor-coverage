# anchor-coverage

A wrapper around [`anchor test`] for computing test coverage

## Steps to use

1. Install the Agave validator [from source] after adding the following to the `[patch.crate-io]` section near the end of its Cargo.toml:

   ```toml
   solana-sbpf = { git = "https://github.com/trail-of-forks/sbpf-coverage" }
   ```

   For many situations, the following commands should suffice:

   ```sh
   sed -i '/^\[patch\.crates-io\]$/a solana-sbpf = { git = "https://github.com/trail-of-forks/sbpf-coverage" }' Cargo.toml
   ./scripts/cargo-install-all.sh .
   export PATH=$PWD/bin:$PATH
   ```

2. Add the following to `[profile.release]` section of your Anchor project's root Cargo.toml:

   ```toml
   debug = true
   ```

   This tells Cargo to build with debug information.

3. Run `anchor-coverage` as follows:

   ```sh
   anchor-coverage [ANCHOR_TEST_ARGS]...
   ```

   This will create an `sbf_trace_dir` directory with an LCOV file for each executable run.

4. Run the following command to generate and open an HTML coverage report:

   ```sh
   genhtml --output-directory coverage sbf_trace_dir/*.lcov && open coverage/index.html
   ```

## Known problems

`anchor-coverage` uses Dwarf debug information, not [LLVM instrumentation-based coverage], to map instructions to source code locations. This can have confusing implications. For example:

- one line can appear directly before another
- the latter line can have a greater number of hits

The reason is that multiple instructions can map to the same source line. If multiple instructions map to the latter source line, it can have a greater number of hits than the former.

The following is an example. The line with the assignment to `signer` is hit only once. But the immediately following line is hit multiple times, because the instructions that map to it are interspersed with instructions that map elsewhere.

```rs
            5 :     pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
            1 :         let signer = &ctx.accounts.signer;
            4 :         let pubkey = signer.signer_key().unwrap();
           11 :         msg!("Signer's pubkey is: {}", pubkey);
            1 :         Ok(())
            1 :     }
```

## Troubleshooting

- If you see:
  ```
  Line hits: 0
  ```
  Check that you added `debug = true` to the `[profile.release]` section of your Anchor project's root Cargo.toml.

## Links

- Useful reference re LCOV: https://github.com/linux-test-project/lcov/issues/113#issuecomment-762335134

[LLVM instrumentation-based coverage]: https://llvm.org/docs/CoverageMappingFormat.html
[`anchor test`]: https://www.anchor-lang.com/docs/references/cli#test
[from source]: https://docs.anza.xyz/cli/install#building-from-source
