# anchor-coverage

A wrapper around [`anchor test`] for computing test coverage

## Steps to use

1. Install the Agave validator [from source] after adding the following to the `[patch.crate-io]` section near the end of its Cargo.toml:

   ```toml
   sbpf = { git = "https://github.com/trail-of-forks/sbpf-coverage" }
   ```

2. Ensure the root Cargo.toml of your Anchor project contains the following:

   ```toml
   [profile.release]
   debug = true
   ```

   This tells Cargo to build with debug information.

3. Run `anchor-coverage` as follows:

   ```sh
   anchor-coverage [ANCHOR_TEST_FLAGS]...
   ```

   This will create an `sbf_trace_dir` directory with an LCOV file for each executable run.

4. Run the following command to generate and open an HTML coverage report:

   ```sh
   genhtml --output-directory coverage sbf_trace_dir/*.lcov && open coverage/index.html
   ```

## Links

- Useful reference re LCOV: https://github.com/linux-test-project/lcov/issues/113#issuecomment-762335134

[`anchor test`]: https://www.anchor-lang.com/docs/references/cli#test
[from source]: https://docs.anza.xyz/cli/install#building-from-source
