# rustysd
A minimal drop-in for (a subset of) the systemd init system in rust. For now that is just out of interest how far I could come with this 
and what would be needed to get a somewhat working system. It is very much a proof of concept. For the love of god do not use this
in anything that is important.

It does look somewhat promising, the really ugly parts are "working". There is a lot of cleanup to be done. There is a whole lot of unwrap() calling
where error handling should be done properly.


## What works
This section should be somewhat up to date with what parts are (partly?) implemented

1. Parsing of very simple service files
1. Ordering of services according to the before/after relations
1. Killing services that require dead services 
1. Parsing of very simple socket files that use streaming unix sockets
1. Matching services and sockets by name. Just dbus.service to dbus.socket nothing else yet (but that should not be too difficult)
1. Passing filedescriptors to the daemons
1. Naming file descriptors in the env variables with the name from the *.socket file
1. Waiting for the READY=1 notification
1. Matching services and sockets either by name or dynamically by parsing the appropiate settings in the .service/.socket files
1. The parts of the sd_notify API


### See for yourself
Running `./build_all.sh && cargo run --bin rustysd` will build the test service and run rustysd which will start that testservice

## What does not work
Just some stuff I know does not work but would be cool to have

1. The whole dbus shenanigans
1. More socket types 
    1. fifos are missing
1. The whole sd_notify API (with storing filedescriptors and such)
1. A systemctl equivalent to control/query rustysd 