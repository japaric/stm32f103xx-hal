set -euxo pipefail

main() {
    local src=$(pwd)
    local td=$(mktemp -d)

    pushd $td
    cargo clone cortex-m-quickstart --vers 0.1.8
    cd cortex-m-quickstart

    rm -rf build.rs examples memory.x
    ln -s $src/examples .

    cargo add blue-pill --path $src
    cargo add cortex-m-rtfm --vers 0.1.1

    for path in $(ls examples/*); do
        local ex=$(basename $path)
        ex=${ex%.*}

        xargo check --example $ex --target $TARGET
    done

    popd
    rm -rf $td
}

main
