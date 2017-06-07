set -euxo pipefail

main() {
    cargo install --list | grep xargo || \
        cargo install xargo

    rustup component list | grep 'rust-src.*installed' || \
        rustup component add rust-src
}

main
