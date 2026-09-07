#! /bin/bash

# smoelius: This script tests the fixtures with `anchor test` rather than `anchor-coverage`.

set -euo pipefail

AGAVE_TAG="$(cat agave_tag.txt)"

case "$(uname -s)" in
    Darwin)
        EXT=macOS
        ;;
    Linux)
        EXT=Linux
        ;;
    *)
esac

TOOLS_NAME="patched-agave-tools-$AGAVE_TAG-$EXT"
TOOLS_ARCHIVE="$TOOLS_NAME.tar.gz"
TOOLS_DIR="$PWD/$TOOLS_NAME"

if [[ ! -x "$TOOLS_DIR/bin/solana-test-validator" ]]; then
    if [[ ! -f "$TOOLS_ARCHIVE" ]]; then
        wget --no-verbose \
            "https://github.com/trail-of-forks/sbpf-coverage/releases/download/$AGAVE_TAG/$TOOLS_ARCHIVE"
    fi
    tar xzf "$TOOLS_ARCHIVE"
fi

# smoelius: `anchor-coverage` automatically finds the patched tools, but `anchor test` does not.
# So, add them to `PATH` explicitly.
export PATH="$TOOLS_DIR/bin:$PATH"

for X in fixtures/*; do
    if [[ "$X" = fixtures/retry ]]; then
        continue
    fi

    pushd "$X"

    yarn

    # smoelius: Arguments passed to this script are forwarded to `anchor test`. For example, one
    # can pass `--validator legacy` to use the legacy validator.
    anchor test "$@"

    popd
done
