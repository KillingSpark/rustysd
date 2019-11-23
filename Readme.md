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

## What does not work
Just some stuff I know does not work but would be cool

1. The whole dbus shenanigans
1. Matching services and sockets more dynamically
1. More socket types 
    1. unix sequential sockets ar missing... does any even use these?
    1. fifos are missing
1. The whole sd_notify API (with storing filedescriptors and such)
1. Naming file descriptors in the env variables with the name from the *.socket file

## Boot ordering
Needs dbus for:
1. Start NetworkManager with type=dbus
2. Wait for NetworkManager to grab its dbus name
3. Continue booting

Sockets are activated before socket.target. All services that have a socket unit are assumed to be "up" after the socket exists
If they fail afterwards that poses some trouble.

## Child listening
needs [signalhook](https://github.com/vorner/signal-hook) for signals
the iterator module is probably enough. 
1. listen for sig_chld signals
2. call waitpid(-1,&child_exit_status,WNOHANG) to get child pid and check what the restart policy is
3. restart unit/kill depending units
