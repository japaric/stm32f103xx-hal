set -euxo pipefail

main() {
    cargo check --target $TARGET

    if [ $TARGET != x86_64-unknown-linux-gnu ]; then
        cargo check --target $TARGET --examples
    fi
}

main
