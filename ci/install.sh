set -euxo pipefail

main() {
    if [ $TARGET = thumbv7m-none-eabi ]; then
        rustup target add $TARGET
    fi
}

if [ $TRAVIS_BRANCH != master ]; then
    main
fi
