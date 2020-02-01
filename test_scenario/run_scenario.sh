# This should print the output of the services ordered by the first number
../target/debug/rustysd 2> /dev/null | grep ".service]"