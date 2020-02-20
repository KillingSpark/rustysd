# rustysd
Rustysd is a service manager that tries to replicate systemd behaviour for a subset of the configuration possibilities. It focuses on the core functionality of a service manager, not requiring to be PID1 (aka init process).

## Will this replace systemd?
TLDR: No, rustysd is no dedicated replacement. It is an opportunity for the niches where systemd could not get it's foot down to profit (more easily) from the 
ecosystem around systemd.

Very likely no. There are a lot of reasons, but most importantly: it works and provides features that rustysd will likely never provide. 

This project might be whats needed to show that the core systemd functionality is not very hard to replicate and that all the advantages of 
having a systemd-like service manager can be brought to many other platforms is very much feasible without having to port all of systemd. There are 
some (a lot?) platforms that rust does not (yet) fully support so the maintainers will understandably 
reject using rustysd as their main service manager. But having rustysd as an example might help other efforts in more portable languages.

Rustysd also opens up usage of systemd services outside of systemd based linux distros (like alpine linux, commonly used in docker containers and small vms) and 
freebsd.


## General info

For now this project is just out of interest how far I could come with this 
and what would be needed to get a somewhat working system. It is very much a proof of concept / work in progress. For the love of god do not use this
in anything that is important.

It does look somewhat promising, most needed features are there. There are a lot of tests missing and more care needs to be taken so that rustysd itself never panics.

### Short intro to systemd / rustysd
Systemd/rustysd operate on so called "units". These are smallish separate entities in the system like a single service. These units can be handled independently but 
can specify relations to other units that they "need". The unit of service-abc can say "I need the unit of service-xyz to be started before I do".

The second thing systemd/rustysd bring to the table is socket activation. Services that specify sockets do not need to be started immediately, but rather when there is 
activity on their socket(s). This enables faster startup times by starting services lazily when they are needed.

Additionally systemd provices a lot more unit-types besides services and sockets which rustysd does not (and for most will likely never) support. 


## Scope of rustysd

What is explicitly in scope of this project
1. Startup sorted by dependencies (parallel if possible for unrelated units)
1. Startup synchronization via *.target units
1. Socket activation of services

What is explicitly out of scope (for now, this project is still very young):
1. Timers (Cron should do fine for 99% of usecases)
1. Mounts (It is actually useful to have these as units but I don't think the gains outweigh the added complexity)
1. Device (Same argument as for Mount)
1. Path activation (Might get included.)
1. Scopes (Nope. If you start processes outside of rustysd you need to manage them yourself. Maybe a second instance of rustysd? ;))
1. Slices (this might be added as it is fairly important if you are not running inside of a container)

[![Gitter](https://badges.gitter.im/rustysd/community.svg)](https://gitter.im/rustysd/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

### About slices
I dont think it is viable for a cross-platform project to support slices. In general I think it would be more sensible to put that responsibility on other tools.

I imagine something along the lines of dockers 'runc' but not specialized to the container environment. Let's call the imaginary tool 'restrict', the usage I 
imagine would be along the lines of: 

`restrict -cmd "/my/binary arg1 arg2 arg3" -mem_max=5G -io_max=10G/s`

This would setup the process with the given restrictions and then exec into the given cmd. with this kind of tool there are a few benefits:
1. Rustysd doesnt have to concern itself with how a platform does resource restriction, but there can be separate tools for each platform (if possible)
1. Clear separation of concerns. Rustysd manages service lifetimes. It does not (or only for relatively trivial stuff) manage the runtime environment for those services.
1. The tool can be useful to other contexts aswell  

## Goals
Since this project is very young and wasn't started with any particular goal in mind, I am open to any and all ideas. Here are some that have been 
brought up that seem sensible. None of this is definitive though.

1. Provide a PID1 for containers/jails, so unaltered systemd depending services can be run in a container/jail
1. Provide full init capabilities so this can be used for OS's like redox os or debian/kFreeBSD
1. Be platform agnostic as long as it's unix (I develop on linux but I'll try to test it on FreeBSD when I add new platform specific stuff)

### About platform independence
Here is a list of features rustysd currently assumes the platform to provide. The list also contains suggestions about which features
could be cut and which consequence that would have (e.g. see the filedescriptor point). Everything here is written in unix terms but it should not be too much work
to write a compatibility shim if an equivalents exist on the target platform. It's not too many features that must exist for the port to work in a usable way (and they are mostly basic OS functionality anyways).
1. forking
1. getting the current process id
1. file descriptors that can be passed to child processes when forking
    * Maybe we dont have to have this. We could just make sockets and socket-activation an optional feature for unixy platforms
    * Then forking would be optional too, just having the ability to launch new executables in a new process would suffice
1. (Un-)Mark file descriptors for closing on exec()'ing if forking with passed fds is supported
1. Select()'ing on filedescriptors (not just for socket activation but for listening on stdout/err of child processes)
1. Creating a pipe/eventfd/... for interrupting the selects (also a way to activate/reset those, write(/read() for pipes for example)
1. dup2()'ing filedescriptors for providing fds at fd index 3,4,5,...
1. Creating process-groups
1. signals from the platform when a child exits / gets terminated in any way
1. waitpid()'ing for the exited children
1. sending (kill/terminating) signals to whole process groups (as long as we care about cleanup after killing, maybe the platform handles this in another smart way?)
1. setting env variables (currently handled with libc because the rust std contains locks which currently break on forking)
1. setting the current process as a subprocess reaper (might not be that important, other platforms might handle reparenting of orphaned processes differently than unix)
1. changing the user id to drop privileges
1. an implementation of getpwnam_r and getgrnam_r. These can be swapped for getpwnam/getgrnam if needed
    * They could also be ignored, restricting the values of User, Group, and SupplementaryGroups to numerical values

### About platform dependent stuff
There are some parts that are platform dependent. Those are all optional and behind feature flags.

#### Cgroups
Rustysd can employ cgroups for better control over which processes belong to which service. Resource-limiting is still out of scope for rustysd. Cgroups
are only used to make the features rustysd provides anyways more reliable.

On other systems there might arise issues if a service forks of processes which move into another process-group. If these are not cleanly killed by the 
stop/posstop commands they will be orphaned and survive. This is (if I understand correctly) the way other service manager handle this too. 

#### Why no jails
One possibility would be to use BSD jails but that seems somewhat hacky since rustysd would have to chain-load the actual service command with a jail command.
Rustysd could check if a service is started in a jail anyways and then kill that jail. But that could lead to other problems if the jail is meant to be long lived.
In short I see no clean way to employ jails for process-management

## What works
This section should be somewhat up to date with what parts are (partly?) implemented and (partly?) tested. If you find anything does actually not work
please file me an issue! 

For an in-depth comparison of systemd and rustysd see the feature-comparison.md file. It is generated by the tools/gen_feature_comparison.py (shoutout to [wmanley](https://github.com/wmanley) who wrote the initial script!).
It currently is somewhat pessimistic, I will work on improving the comparison of the features rustysd actually does support (see below for a list of supported features).

### General features
Of rustysd itself
* Parsing of service files (a subset of the settings are recognized)
* Parsing of socket files (a subset of the settings are recognized)
* Ordering of services according to the before/after relations
* Killing services that require services that have died 
* Matching services and sockets either by name or dynamically by parsing the appropiate settings in the .service/.socket files
* Passing filedescriptors to the daemons as systemd clients expect them (names and all that good stuff)
* Pretty much all parts of the sd_notify API
* Waiting for the READY=1 notification for services of type notify
* Waiting for services of type dbus
* Waiting for multiple dependencies
* Target units to synchronize the startup
* Send SIGKILL to whole processgroup when killing a service
* Socket activation (the non-inetd style). So your startup will be very fast and services only spin up if the socket is actually activated
* Pruning the set of loaded units to only the needed ones to reach the target unit

With the control interface (doc/ControlInterface.md for a detailed list of commands) 
* Adding new units while running
* Restarting units
* Stopping units
* Shutdown rustysd

### Optional build features
There are some features behind flags because they are either platform dependent or not necessarily needed for most of the use-cases
* dbus_support: Activate support for services of type dbus (not needed for many services and probably a dumb idea in a container anyways)
* linux_eventfd: Use eventfds instead of pipes to interrupt select() calls (because they only exist on linux)
* cgroups: Optional support to use cgroups to more reliably kill processes of services on linux

### Docker
Running in a docker container as PID1 works. The image that is built by the scripts in the dockerfiles directory results in a ~2MB image that contains
* Rustysd (stripped binary built with musl to be completely static) -> ~1.6Mb
* The testservice and testserviceclient (stripped binaries built with musl to be completely static) -> ~300kb / ~280kb
* The unit files in test_units


### See for yourself
Running `./build_all.sh && cargo run --bin rustysd` will build the test services and run rustysd which will start them.
Currently there are two services, one that gets passed some sockets and one that uses them to send some text over those sockets.

There are some scripts to run this in a docker container. Have a look at the scripts in the dockerfiles directory.

## What does not work
Just some stuff I know does not work but would be cool to have. I tried to categorize them by how much work the seem to be, but otherwise they
are without a particular oder.

Requiring bigger changes or seem complicated:
* Unit templates
* An optional journald logging. (Maybe thats not something that is actually something that is wanted)
    1. Positive: Better compatibility
    1. Negative: Weird dependency between rustysd and a service managed by rustysd (could be less of a pain point if rustysd itself handled logging in a journald way)
* Socket activation in inetd style
* The whole dbus shenanigans (besides waiting on dbus services, which is implemented)
* Service type forking is missing
* The rest of the sd_notify API (with storing filedescriptors and such)

Requiring small changes / additions transparent to the other modules:
* Change user to drop privileges
* Patching unit definitions with dropin files
* Socket options like MaxConnections=/KeepAlive=
* Killing services with a configurable signal. Currently its always SIGKILL after the ExecStop commands have been run
* More socket types 
    1. Netlink is missing for example
    1. Abstract namespace for unix sockets (but thats linux specific anyways and rust stdlib doesnt support it.....)
* Service type idle is missing (not even sure if its a good idea to support this)
* A systemctl equivalent to control/query rustysd (there is a small jsonrpc2 API but that might change again)
    * Disabling of units is missing
    * A better UI than pretty-printed json is missing
* Many of the missing features in feature-comparison.md are relatively simple issues
* Support the different allowed prefixed for executables in execstart(-pre/post)

Unclear how much work it is:
* Get all the meta-targets and default dependencies right
    * Individually these are probably small parts. But as a whole task it seems like much


## What could be done better
Some stuff where I chose something along the way where there might be better/other choices

1. Use mio instead of nix::select to get events from the stdout/stderr/notification-sockets
    1. Pro: uses more modern/efficient APIs (epoll/kqeueu)
    1. Con: Probably less portable to more exotic unices (like redox)

## How does it work
Rustysd has two binaries: The main service-manager 'rustysd' and the control client 'rsdctl'. 

The client is just a dumb utility to 
pack cli arguments into jsonrpc2 format and send them to rustysd. This can be used to restart units, add new units, or show the status
of units. 

Generally rustysd has two phases:
1. Bring up all units with as much concurrency as possible, and as lazily (with socket activation) as possible
2. Wait for events from the services or the control sockets, and react to these
    1. Data from either stdout/err or the notification sockets
    2. Signals from the kernel

## Community
There has been a request for a place to talk about this project, so I opened a gitter community for this project. Feel free to come over and have a chat [on this page](https://gitter.im/rustysd/community?utm_source=share-link&utm_medium=link&utm_campaign=share-link)
