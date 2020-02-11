#! /bin/bash
# This should print the output of the services ordered by the first number
../target/debug/rustysd 2> /dev/null | grep ".service]" &

sleep 5
killall rustysd
sleep 1