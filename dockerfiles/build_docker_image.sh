#! /bin/sh
cargo build --target x86_64-unknown-linux-musl --release
cd target/x86_64-unknown-linux-musl/release
strip --strip-unneeded rustysd
strip --strip-unneeded testservice
strip --strip-unneeded testserviceclient
cd ../../..
docker build -t killingspark/rustysdtest -f dockerfiles/Dockerfile .
