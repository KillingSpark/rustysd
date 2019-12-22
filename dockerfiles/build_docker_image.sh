#! /bin/sh
cargo build --target x86_64-unknown-linux-musl
docker build -t killingspark/rustysdtest -f dockerfiles/Dockerfile .
