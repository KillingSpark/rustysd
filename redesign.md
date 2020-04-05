# Redesign
This document tracks the new design before the rewrite [(tracking issue)](https://github.com/KillingSpark/rustysd/issues/35) starts.

## Why this is needed 
Currently rustysd is still pretty spaghetti, since I just started writing code to see what components are needed, and wanted to play around with this project. Now that pretty much all concepts work(tm) the existing bugs are likely caused by the bad (read: missing) design.

Much code concerning the starting/stopping of single units can be reused, but the code around handling a dynamic set of units has to be rewritten.

## General thoughts on what is good/bad currently

Things that need to be particularly looked out for while designing:
1. Separate 'static' info about units like names/config from the 'runtime' info like status, pid, open fds,...
1. Locking. Mutexes are something I first started using when starting rustysd so I used them all over the place and by that enabled deadlocking to happen. This shouldbe possible to work around by making rules in which order stuff has to be locked.
1. Updating the set of units needs to happen 'atomically' so we need to be able to lock the whole runtimeinfo
1. Make finding units and their inter-dependencies easier. It is currently very annoying and verbose to find all units that need each other either by name or implicit dependency. This probably means rustysd should keep track of the name-dependencies when adding/removing units.
1. From the beginning, keep in mind that the set of units is neither static nor that units will keep existing if they existed once. This is one of the biggest issues in the current codebase.

Things that I will probably keep conceptually:
1. The RuntimeInfo struct worked pretty well
1. The FdStore is nice
1. The whole config parsing stuff is ok. It needs to output different types though, since the info organization in the different sections is less than ideal

Things that likely will change:
1. Dont use u64 for IDs. Just use the names of the units, those have to be unique anyways. And the performance of comparing IDs shouldn't really matter. This is 
    an obvious case of premature optimiziation that made the code and tracing unnecessarily weird.

## The new design
The Unit structures need refactoring. The goal should look like this:

1. Rustysd knows one RwLock<UnitSet>.
1. The Units in this UnitSet are not wrapped in separate RwLocks
1. Units structures:
    * Start/Stop dependencies generated from the configs (bidirectional can be implicit if the other unit mentions this one explicitly)
    * Remove dependencies generated from the configs (unidirectional, explicit mentions by name)
    * Immutable config (specialized for the different types) like ExecStart, ExecStop, Environment, FiledescriptorName, ...
    * RwLock around Mutable state (specialized for the different types) like pid, stdio fds, socket fds, ...
    * RwLock around the unit status

The status needs it's own RwLock so other threads can check the status of the unit while it is being started/stopped. Else checking of a units status 
can be blocked by (for example) long running ExecPreStart processes. 

The unitset exposes functions to start/stop/add/remove units which deal with the whole dependency walking. The units themselves will only deal with their
own stuff. E.g. when a service unit gets told to start it starts, without checking dependencies again. This needs special care to work properly though. See the
`Locking` paragraph.

To start a unit only the Immutable config and the mutable state are necessary (additionally the fdstore is necessary too).

### Locking
This section contains all info on how locking needs to be done to avoid deadlocks and ensure correctness of the unit status at all times. 

#### Changing a single units status (starting / stopping units)
To change the status of a unit (start/stop) 
1. the whole unitset needs to be locked at least read()
1. the mutable state needs to be locked write()
1. the status needs sometimes be locked write() to update it

The last two need special care so the status always represents the current status of the unit. If the mutable state is to be locked write() the following
steps need to be followed:
1. Lock the status write()
1. check that nothing else is currently working on the unit (e.g. status == Status::Starting)
    1. If something else is currently working on it stop whetever you tried to do with an appropriate error or try again at a lter point 
        (maybe there should be a condvar for each unit that can be waited on, which is signaled when the unit goes from starting -> started?)
    1. If nothing is currently working on the unit and the status is the expected one (stopping a unit needs it to be in the status started) go 
        forward with whatever you wanted to do
1. update the status to the appropiate status that signals that the unit is in the process of changing its status (e.g. status = Status::Stopping)
1. DROP THE STATUS LOCK HERE
1. Lock the mutable state write() and proceed with your operation

Once the mutable status is locked write() the status lock can be reaquired freely to update the status to one that signals the unit os no longer beeing worked on.

If the status got updated after the unit is locked this bad scenario could happen:
1. Thread 1 locks unit A to stop it
1. Thread 2 checks status to check if it is ok to start unit B
1. Thread 3 updates status of A to stopping

Now the unit B was started even though unit A was stopped, which should not happen if B requires A.

#### Changing the unit set (adding / removing units)
To change the set of units the unitset needs to be locked write() which means while this runs no changes to units may happen.
This sounds a bit limiting but it is what is needed to reliably answer the question 'is it currently legal to remove this set of units'.

### Events
Events are what rustysd should work on. Currently it only does so after the initial startup, which is bad because it duplicates a lot of code. And 
only the intial startup does actually start units in parallel.

Events are one of these:
1. A unix signal was received
1. A command was sent over the control interface to stop/start/add/remove units
1. A socket of a service that was currently waiting for socket-activation was activated

Initially rustysd has a set of inactive units. Then a command will be emulated that starts the configured target unit (default.target in most cases).
This will trigger a recursive (and where possible parallel!) startup of all units that should be started to reach that target.