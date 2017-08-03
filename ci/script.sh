set -euxo pipefail

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo check --target $TARGET
        return
    fi

    xargo check --target $TARGET
    xargo test --target $TARGET --examples
}

main
