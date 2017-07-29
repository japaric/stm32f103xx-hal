set -euxo pipefail

main() {
    if [ $TARGET = thumbv7m-none-eabi ]; then
        local vers=0.3.7

        cargo install --list | grep "xargo v$vers" || \
            ( cd .. && cargo install xargo -f --vers $vers )

        rustup component list | grep 'rust-src.*installed' || \
            rustup component add rust-src
    fi
}

main
