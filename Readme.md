# rustysd
A minimal drop-in for (a subset of) the systemd init system in rust. For now that is just out of interest how far I could come with this 
and what would be needed to get a somewhat working system. It is very much a proof of concept. For the love of god do not use this
in anything that is important.

It does look somewhat promising, the really ugly parts are "working". There is a lot of cleanup to be done. There is a whole lot of unwrap() calling
where error handling should be done properly.

## What works
This section should be somewhat up to date with what parts are (partly?) implemented

1. Parsing of service files (a subset of the settings are recognized)
1. Parsing of socket files (a subset of the settings are recognized)
1. Ordering of services according to the before/after relations
1. Killing services that require services that have died 
1. Matching services and sockets either by name or dynamically by parsing the appropiate settings in the .service/.socket files
1. Passing filedescriptors to the daemons as systemd clients expect them (names and all that good stuff)
1. Pretty much all parts of the sd_notify API
1. Waiting for the READY=1 notification for services of type notify
1. Waiting for services of type dbus
1. Waiting for multiple dependencies
1. Target units to synchronize the startup


### See for yourself
Running `./build_all.sh && cargo run --bin rustysd` will build the test service and run rustysd which will start that testservice

## What does not work
Just some stuff I know does not work but would be cool to have
1. Pruning the set of loaded units to only the needed one to reach the target unit
1. Socket activation. Right now services with a socket will be started right away in parallel with all others. They could just not be spawned and waited on with a select until they are ready. This will be needed anyways for inetd style socket activation
1. Socket options like MaxConnections=/KeepAlive=
1. Killing services properly. SigTerm/Kill/Hup/ executing the stop commands .....
1. The whole dbus shenanigans (besides waiting on dbus services, which is implemented)
1. More socket types 
    1. Netlink is missing for example
1. The rest of the sd_notify API (with storing filedescriptors and such)
1. A systemctl equivalent to control/query rustysd (some querying has been implemented using serde-json but its just a concept right now)