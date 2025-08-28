mod? local

export RUST_TOOLCHAIN := env_var_or_default('RUST_TOOLCHAIN', 'nightly')

fmt *args:
    cargo +{{RUST_TOOLCHAIN}} fmt --all {{args}}

lint *args:
    cargo clippy --fix --allow-dirty --allow-staged --all-features --all-targets {{args}}

alias coverage := test
test *args:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -x ./temp/chromedriver ]; then
        ./temp/chromedriver --port=4444 &
        chromedriver_pid=$!
    else
        chromedriver --port=4444 &
        chromedriver_pid=$!
    fi
    trap "kill $chromedriver_pid" EXIT

    if [ -f target/wasm32-wasip1/release/mangaread_org.wasm ]; then
        export VAULT_TEST_WASM_PLUGIN="$(pwd)/target/wasm32-wasip1/release/mangaread_org.wasm"
        export VAULT_TEST_WASM_PLUGIN_MANGA_URL="https://www.mangaread.org/manga/solo-leveling-manhwa/"
    elif [ -f target/wasm32-wasip1/release/hari_manga.wasm ]; then
        export VAULT_TEST_WASM_PLUGIN="$(pwd)/target/wasm32-wasip1/release/hari_manga.wasm"
        export VAULT_TEST_WASM_PLUGIN_MANGA_URL="https://harimanga.com/manga/solo-leveling/"
    elif [ -f target/wasm32-wasip1/release/manga_dex.wasm ]; then
        export VAULT_TEST_WASM_PLUGIN="$(pwd)/target/wasm32-wasip1/release/manga_dex.wasm"
        export VAULT_TEST_WASM_PLUGIN_MANGA_URL="https://mangadex.org/title/32d76d19-8a05-4db0-9fc2-e0b0648fe9d0/solo-leveling"
    fi

    INSTA_FORCE_PASS=1 cargo test --workspace --all-features {{args}}


