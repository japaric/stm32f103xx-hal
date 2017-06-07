set -euxo pipefail

main() {
    local vers=0.3.7

    cargo install --list | grep "xargo v$vers" || \
        cargo install xargo -f --vers $vers

    cargo install --list | grep cargo-clone || \
        cargo install cargo-clone

    rustup component list | grep 'rust-src.*installed' || \
        rustup component add rust-src
}

main
