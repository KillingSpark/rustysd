# rustysd
Rustysd is a service manager that tries to replicate systemd behaviour for a subset of the configuration possibilities. It focuses on the core functionality of a service manager.

For now this project is just out of interest how far I could come with this 
and what would be needed to get a somewhat working system. It is very much a proof of concept / work in progress. For the love of god do not use this
in anything that is important.
It does look somewhat promising, the core parts are "working" (not thoroughly tested) but there is a lot of cleanup to be done. There is a whole lot of unwrap() calling
where error handling should be done properly. It would be a bit unhelpful if your service-manager starts panicing.

What is explicitly in scope of this project
1. Startup sorted by dependencies (parallel if possible for unrelated services)
1. Socket activation of services
1. Kill services that have dependencies on failed services

What is explicitly out of scope (for now, this project is still very young):
1. Timers
1. Mounts
1. Device
1. Path activation
1. Scopes
1. Slices (this might be added as it is fairly important if you are not running inside of a container)

[![Gitter](https://badges.gitter.im/rustysd/community.svg)](https://gitter.im/rustysd/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

## Goals
Since this project is very young and wasn't started with any particular goal in mind, I am open to any and all ideas. Here are some that have been 
brought up that seem sensible. None of this is definitive though.

1. Provide a PID1 for containers/jails, so unaltered systemd depending services can be run in a container/jail
1. Provide full init capabilities so this can be used for OS's like redox os or debian/kFreeBSD
1. Be platform agnostic as long as it's unix (I develop on linux but I'll try to test it on FreeBSD when I add new platform specific stuff)

## What works
This section should be somewhat up to date with what parts are (partly?) implemented and (partly?) tested

### General features

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
1. Send SIGKILL to whole processgroup when killing a service
1. Socket activation (the non-inetd style). So your startup will be very fast and services only spin up if the socket is actually activated
1. Pruning the set of loaded units to only the needed one to reach the target unit (right now it's not pruned to the actual minimum, but no needed units are removed. Sockets for example are all being kept around right now)

### Optional build features
There are some features behind flags because they are either platform dependent or not necessarily needed for most of the use-cases
1. dbus_support: Activate support for services of type dbus (not needed for many services and probably a dumb idea in a container anyways)
1. linux_eventfd: Use eventfds instead of pipes to interrupt select() calls (because they only exist on linux)

### Docker
Running in a docker container as PID1 works. The image that is built by the scripts in the dockerfiles directory result in a 2MB image that contains
1. Rustysd (stripped binary built with musl to be completely static)
1. The testservice and testserviceclient (stripped binaries built with musl to be completely static)
1. The unit files in test_units


### See for yourself
Running `./build_all.sh && cargo run --bin rustysd` will build the test services and run rustysd which will start them.
Currently there are two services, one that gets passed some sockets and one that uses them to send some text over those sockets.

## What does not work
Just some stuff I know does not work but would be cool to have.
1. Better pruning of the units to reach the target unit
1. Get all the meta-targets and default dependencies right
1. Unit templates
1. Patching unit definitions with dropin files
1. Timeouts for service starting
1. Socket activation in inetd style
1. Socket options like MaxConnections=/KeepAlive=
1. Killing services properly. SigTerm/Kill/Hup/ executing the stop commands .....
1. The whole dbus shenanigans (besides waiting on dbus services, which is implemented)
1. More socket types 
    1. Netlink is missing for example
1. More Service types 
    1. forking is missing
    1. oneshot is missing
    1. idle is missing
1. The rest of the sd_notify API (with storing filedescriptors and such)
1. A systemctl equivalent to control/query rustysd (there is a small jsonrpc2 API but that might change again)

## How does it work
Generally rustysd has two phases:
1. Bring up all units with as much concurrency as possible, and as lazily (with socket activation) as possible
2. Wait for events from the services, and reat to these
    1. Data from either stdout/err or the notification sockets
    2. Signals from the kernel

## Community
There has been a request for a place to talk about this project, so I opened a gitter community for this project. Feel free to come over and have a chat [on this page](https://gitter.im/rustysd/community?utm_source=share-link&utm_medium=link&utm_campaign=share-link)
