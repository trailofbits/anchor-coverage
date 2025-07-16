#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

if [[ ! -v AGAVE ]]; then
    echo "$0: expect AGAVE to be set to the path of an agave repository checked out at commit 7a753db" >&2
    exit 1
fi

git clone https://github.com/anza-xyz/rust
cd rust
git submodule update --init
cd src/llvm-project
# smoelius: The commit below is anza-xyz/llvm-project#159. Git must be configured before
# `git cherry-pick` can be called.
# git config --global user.email "you@example.com"
# git config --global user.name "Your Name"
git fetch origin ecdfcf877b5053e7d85d9102118f59b64e020432
git cherry-pick ecdfcf877b5053e7d85d9102118f59b64e020432
cd ../.. # rust
git add src/llvm-project
git commit -m 'Fix debug relocation'
./build.sh
# smoelius: Install platform tools v1.49.
"$AGAVE"/platform-tools-sdk/sbf/scripts/install.sh
RUST="$PWD"
cd ~/.cache/solana/v1.49/platform-tools/rust/bin
mv rustc rustc~
ln -s "$RUST"/build/x86_64-unknown-linux-gnu/stage1/bin/rustc
