#! /bin/sh
cargo build
# cargo build --features linux_eventfd
# cargo build --target x86_64-unknown-redox

cd test_c_service
clang -o test_service test_service.c $(pkg-config libsystemd --libs)