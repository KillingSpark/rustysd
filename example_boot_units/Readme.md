# Collection of boot units
This is just a try to collect units that would allow a pretty generic system to boot. The general idea is:

1. Mount all local (non-network) fielsystems with mount -a
1. Start the network interfaces (missing)
1. Mount all network shares with mount -a
1. Start gettys on a few ttys 

I have yet to try this in a VM. There are probably more steps necessary. (Convert the [rc.boot](https://github.com/kisslinux/init/blob/master/lib/init/rc.boot) from Kiss linux?)