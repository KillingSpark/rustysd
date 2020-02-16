# Stdio handling
This document gives a overview of how stdio filedesscriptors are passed around in rustysd

## Of services
There are two pipes opened for every service. One for stdout and stderr. When the service is started these are put at FDs 1 and 2 with dup2().

In src/notification_handler are two 'select()' calls, one that selects on all stdouts of all services and one for the stderrs. If the selects return the pipes 
will be read until they would block again and the select is called again. 

The content is buffered and only output if a line separator ('\n') or a zero byte ('\0') is encountered.

Currently the output of services is just printed on stdout/err of rustysd with a prefix that 
identifies the service but in the future this should be incorporated somehow into the logging solution used.

## Of ExecStartPre/-Post and ExecStop(-Post)
These are usually short commands with only few lines of output. Here the rusts stdlib is used to just collect all output and collect it after the process exits.
It is then handled just like the output of the normal service executable