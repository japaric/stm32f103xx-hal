set -euxo pipefail

main() {
    cargo install --list | grep xargo || \
        cargo install xargo

    cargo install --list | grep cargo-clone || \
        cargo install cargo-clone

    rustup component list | grep 'rust-src.*installed' || \
        rustup component add rust-src
}

main
