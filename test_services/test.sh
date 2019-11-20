#! /bin/bash

echo "FROM SERVICE: FDS:" $LISTEN_FDS "PID: " $LISTEN_PID

cat <&3

echo "WROTE TO FD 3"