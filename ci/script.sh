set -euxo pipefail

main() {
    cargo check --target $TARGET

    if [ $TARGET != x86_64-unknown-linux-gnu ]; then
        # fast check (it compiles)
        cargo check --target $TARGET --examples

        # it links (using release because some programs don't fit in Flash when unoptimized)
        cargo build --target $TARGET --examples --release
    fi
}

if [ $TRAVIS_BRANCH != master ]; then
    main
fi
