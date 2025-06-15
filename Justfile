mod? local

# By default we use the nightly toolchain, however you can override this by setting the RUST_TOOLCHAIN environment variable.
export RUST_TOOLCHAIN := env_var_or_default('RUST_TOOLCHAIN', 'nightly')

# An alias for cargo fmt --all
fmt *args:
    cargo +{{RUST_TOOLCHAIN}} fmt --all {{args}}

lint *args:
    cargo clippy --fix --allow-dirty --allow-staged --all-features --all-targets {{args}}

alias coverage := test
test *args:
    #!/usr/bin/env bash
    set -euo pipefail
    
    ./temp/chromedriver --port=4444 &
    chromedriver_pid=$!
    trap "kill $chromedriver_pid" EXIT
    
    INSTA_FORCE_PASS=1 cargo +{{RUST_TOOLCHAIN}} llvm-cov clean --workspace
    INSTA_FORCE_PASS=1 cargo +{{RUST_TOOLCHAIN}} llvm-cov nextest --workspace --include-build-script --no-report --all-features -- {{args}}
    # Coverage for doctests is currently broken in llvm-cov.
    # Once it fully works we can add the `--doctests` flag to the test and report command again.
    cargo +{{RUST_TOOLCHAIN}} llvm-cov test --workspace --doc --no-report --all-features {{args}}

    # Do not generate the coverage report on CI
    cargo insta review
    cargo +{{RUST_TOOLCHAIN}} llvm-cov report --lcov --output-path ./lcov.info
    cargo +{{RUST_TOOLCHAIN}} llvm-cov report --html

coverage-serve:
    miniserve target/llvm-cov/html --index index.html --port 3000

update-scrapers:
    python3 ./scrapers/compile-scrapers.py

update-wit-bindings:
    wit-bindgen rust scraper.wit --format --ownership owning --out-dir scrapers/scraper_types/src/
