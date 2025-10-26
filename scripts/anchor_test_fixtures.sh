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

for X in fixtures/*; do
    if [[ "$X" = fixtures/retry ]]; then
        continue
    fi

    pushd "$X"

    yarn

    wget --quiet https://github.com/trail-of-forks/sbpf-coverage/releases/download/$AGAVE_TAG/patched-agave-tools-$AGAVE_TAG-$EXT.tar.gz

    tar xzf patched-agave-tools-$AGAVE_TAG-$EXT.tar.gz

    # smoelius: `anchor-coverage` will find and use the patched tools, but `anchor test` will not.
    PATH="$PWD/patched-agave-tools-$AGAVE_TAG-$EXT/bin:$PATH"

    anchor test

    popd
done
