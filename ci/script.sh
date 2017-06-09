set -euxo pipefail

main() {
    local src=$(pwd)
    local td=$(mktemp -d)
    local version=0.1.8
    local url=https://github.com/japaric/cortex-m-quickstart/archive/v$version.tar.gz

    pushd $td

    curl -L $url | tar --strip-components 1 -xz

    rm -rf build.rs examples memory.x src
    ln -s $src/examples .

    cat >>Cargo.toml <<EOF
[dependencies.blue-pill]
path = "$src"

[dependencies.cortex-m-hal]
git = "https://github.com/japaric/cortex-m-hal"
rev = "b12039609c6ac2d21392d44cd7080072c509"

[dependencies.cortex-m-rtfm]
version = "0.1.1"

[dependencies.futures]
default-features = false
version = "0.1.14"

[dependencies.nb]
git = "https://github.com/japaric/nb"
EOF

    for path in $(ls examples/*); do
        local ex=$(basename $path)
        ex=${ex%.*}

        case $ex in
            *-await)
                continue
                ;;
        esac

        xargo check --example $ex --target $TARGET
    done

    popd
    rm -rf $td
}

main
