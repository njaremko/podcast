set -ex

build_release() {
    cargo build --target $TARGET --release --verbose
}

mk_tarball() {
    local tmpdir="$(mktemp -d)"
    local name="$PROJECT_NAME-$TRAVIS_TAG-$TARGET"
    local release_dir="$tmpdir/$name"

    mkdir -p "$release_dir"

    cp "target/$TARGET/release/podcast" "$release_dir/podcast"

    cd "$tmpdir"
    tar czf "$TRAVIS_BUILD_DIR/$name.tar.gz" "$name"
    rm -rf "$tmpdir"
}

main() {
    build_release
    mk_tarball
}

main
