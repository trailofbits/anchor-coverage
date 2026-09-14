# anchor-coverage

A wrapper around [`anchor test`] for computing test coverage

`anchor-coverage` requires a [patched] `solana-test-validator` (see below). The patch is known to work with [Agave v3.1.0-beta.0](https://github.com/anza-xyz/agave/tree/v3.1.0-beta.0).

## Steps to use

1. Download, unzip, and untar a patched `solana-test-validator` from `sbpf-coverage`'s [Releases].

2. Add the following to the `[profile.release]` section of your Anchor project's root Cargo.toml:

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
           11 :         msg!("Signer's pubkey: {}", pubkey);
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

- Useful reference re LCOV: [gifnksm/lcov/src/record/mod.rs#L24-L206](https://github.com/gifnksm/lcov/blob/ee7e052aa8bd32c8863edc6f728a1e6f3ad1aa96/lcov/src/record/mod.rs#L24-L206)

[Agave repository]: https://github.com/anza-xyz/agave
[LLVM instrumentation-based coverage]: https://llvm.org/docs/CoverageMappingFormat.html
[Releases]: https://github.com/trail-of-forks/sbpf-coverage/releases
[`anchor test`]: https://www.anchor-lang.com/docs/references/cli#test
[from source]: https://docs.anza.xyz/cli/install#building-from-source
[patched]: https://github.com/trail-of-forks/sbpf-coverage
