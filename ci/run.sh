#!/bin/sh
set -exu
case "$1" in
    build)
        cargo build --verbose
        cargo build --verbose --examples
        ;;
    test)
        cargo test --verbose
        cargo test --verbose -- --ignored
        ;;
    common)
        cd sn0int-common
        cargo test --verbose
        ;;
    std)
        cd sn0int-std
        cargo test --verbose
        cargo test --verbose -- --ignored
        ;;
    windows)
        cargo build --verbose --features=sqlite-bundled
        cargo build --verbose --examples --features=sqlite-bundled
        ;;
    boxxy)
        cargo build --verbose --examples
        if cat ci/boxxy_stage1.txt | RUST_LOG=boxxy cargo run --example boxxy; then
            echo "SANDOX ERROR: should've crashed"
            exit 1
        fi
        ;;
    docker)
        docker build -t sn0int .
        docker images
        docker run --rm sn0int --help
        ;;
    docker-registry)
        docker build -t sn0int-registry -f sn0int-registry/Dockerfile .
        docker images
        ;;
esac
