mod? local

export RUST_TOOLCHAIN := env_var_or_default('RUST_TOOLCHAIN', 'nightly')

fmt *args:
    tools/dprint.dotslash fmt {{args}}

lint *args:
    cargo clippy --fix --allow-dirty --allow-staged --all-features --all-targets {{args}}
    cd apps/website && bun run lint

alias coverage := test
test *args:
    #!/usr/bin/env bash
    set -euo pipefail

    ./tools/chromedriver.dotslash --port=4444 &
    chromedriver_pid=$!
    trap "kill $chromedriver_pid" EXIT

    if [ -f target/wasm32-wasip1/release/mangaread_org.wasm ]; then
        export VAULT_TEST_WASM_PLUGIN="$(pwd)/target/wasm32-wasip1/release/mangaread_org.wasm"
        export VAULT_TEST_WASM_PLUGIN_MANGA_URL="https://www.mangaread.org/manga/solo-leveling-manhwa/"
    else
        echo "WASM plugin not found, building..."
        cargo component build -p mangaread_org --target wasm32-wasip1 --release
        export VAULT_TEST_WASM_PLUGIN="$(pwd)/target/wasm32-wasip1/release/mangaread_org.wasm"
        export VAULT_TEST_WASM_PLUGIN_MANGA_URL="https://www.mangaread.org/manga/solo-leveling-manhwa/"
    fi

    export LUA_PLUGIN_TEST_DIR="$(pwd)/scrapers/"

    cargo test --workspace --all-features {{args}}


