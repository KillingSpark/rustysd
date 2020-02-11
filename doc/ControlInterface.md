# Control Interface
The control-interface provides access similar to systemctl for systemd. It uses the jsonrpc 2.0 spec and has the interface as defined below.



## Call list
This lists all calls possible and their parameters to the control interface. The call are described in detail below

Summary:
| Call name  | args                      |
|------------|---------------------------|
| list-units | optional string 'kind'    |
| status     | optional string 'name'    |
| restart    | string 'name'             |
| stop       | string 'name'             |
| enable     | [string] or string 'name' |
| enable     | [string] 'name'           |
| shutdown   | none                      |
| reload     | none                      |


### CALL: list-units
Args:
1. optional string 'kind'

Notes:
* Kind either "target", "socket", "service"
* Give no kind to list all units of all types
* Lists all units. In the future there should be a filtering mechanism for type / name-matching / etc...

### CALL: status
Args:
1. optional string 'name'

Notes:
* If the param is a string show status of the unit with that name (might get the same filtering as list-units in the future).
* If no param is given, show status of all units

### CALL: restart
Args:
1. string name

Notes:
* Restart unit with that name. If it was running first kill it. If it is already stopped start it.

### CALL: stop
Args:
1. string name

Notes:
* Stop unit with that name. Will recursivly stop all units that require that unit

### CALL: enable
Args:
1. [string] names

Notes:
* Load new file with those name(s). Useful if you moved/copied a file in the unit-dirs and want to start it without restarting rustysd as a whole.
* Note that already loaded units can't be enabled.

### CALL: shutdown
Args:
1. none

Notes:
* Shutdown rustysd by killing all services, closing all sockets and exiting

### CALL: reload
Args:
1. none

Notes:
Reloads all units and adds new ones. Units that are already loaded are ignored. The command responds which units got added and ignored.

## Send commands
There is rsdctl in `src/bin/rsdctl.rs`. This is just a wrapper that converts cli args to jsonrpc calls and send them to a tcp or unix socket.

Alteratively you can use something like socat to send commands or whatever you'd like. (There is a need for a better userinterface though PRs very welcome!)
`echo '{"method": "restart", "params": "test.service"}' | socat - TCP-CONNECT:0.0.0.0:8080`