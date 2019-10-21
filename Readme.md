# rustysd
A minimal drop-in for the systemd init system in rust. For now that is just out of interest how far I could come with this 
and what would be needed to get a somewhat working system

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
