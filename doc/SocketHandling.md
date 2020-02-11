# Socket handling
This document gives a overview of how information about sockets are opened and passed to services / activate services in rustysd.

Sockets are always bound to .socket files. They only get passed to services if either
1. The name matches (e.g. dbus.service <-> dbus.socket)
1. The socket specifies a service to get passed to
1. The service specifies the socket to get passed to it

## Opening
Sockets are opened when the respective .socket unit is activated. This is usually the case early in startup. Most include a "sockets.target" to synchronize startup
at this point. The filedescriptor is stored  in the fd_store field of the RuntimeInfo so services can collect them when they get started.

## Passing to services
When a service starts it collects all needed FDs from the fd_store. After forking they are put at FD 3,4,5... using dup2(). 

All FDs are marked with FD_CLOEXEC so they are closed after execing the service. All FDs needed by the service need this flag to be unset.

## Socket activation
This is currently somewhat bolted on but I am not sure how to do this in a better way. Service units can 'ignore' activation and go into a 'StartedWaitingForSocket' state.
Rustysd has a 'select' waiting on all FDs that are not currently passed to a service. If one of them has data read the respective service is activated (and the possibility to ignore the activation is disabled)