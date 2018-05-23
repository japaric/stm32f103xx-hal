set -euxo pipefail

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        return
    fi

    cargo doc --features doc --target $TARGET

    mkdir ghp-import

    curl -Ls https://github.com/davisp/ghp-import/archive/master.tar.gz | \
        tar --strip-components 1 -C ghp-import -xz

	touch target/$TARGET/doc/.nojekyll
    ./ghp-import/ghp_import.py target/$TARGET/doc

    set +x
    git push -fq https://$GH_TOKEN@github.com/$TRAVIS_REPO_SLUG.git gh-pages && \
        echo OK
}

if [ $TRAVIS_BRANCH = staging ]; then
    main
fi
