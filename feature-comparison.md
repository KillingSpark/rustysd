
# Feature comparison
This document is auto-generated. It pulls all features from the xml-doc from systemd and checks whether the features is supported
in rustysd. (shoutout to [wmanley](https://github.com/wmanley) who wrote the initial script!). Note that this shows a lot 
of crosses. This can have two reasons: 
1. The most likely case is that the feature is not (and will likely never) be supported because it is out of scope of this project (see Readme on how that is determined)
1. The feature is not yet supported but should be. If thats the case please file an issue and I will push it to the top of the priority list.

This document is meant as a simple way of checking whether all features you need from systemd are supported in rustysd.

## sd_notify

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#READY=1">READY=1</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27READY%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Waiting for ready notification for service-type notify is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#RELOADING=1">RELOADING=1</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RELOADING%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#STOPPING=1">STOPPING=1</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27STOPPING%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#STATUS=…">STATUS=…</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27STATUS%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Sending free-text status updates to be displayed for the user is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#ERRNO=…">ERRNO=…</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ERRNO%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#BUSERROR=…">BUSERROR=…</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BUSERROR%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#MAINPID=…">MAINPID=…</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MAINPID%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#WATCHDOG=1">WATCHDOG=1</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WATCHDOG%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#WATCHDOG=trigger">WATCHDOG=trigger</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WATCHDOG%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#WATCHDOG_USEC=…">WATCHDOG_USEC=…</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WATCHDOG_USEC%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#EXTEND_TIMEOUT_USEC=…">EXTEND_TIMEOUT_USEC=…</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27EXTEND_TIMEOUT_USEC%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#FDSTORE=1">FDSTORE=1</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27FDSTORE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#FDSTOREREMOVE=1">FDSTOREREMOVE=1</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27FDSTOREREMOVE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#FDNAME=…">FDNAME=…</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27FDNAME%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/sd_notify.html#$NOTIFY_SOCKET">$NOTIFY_SOCKET</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27NOTIFY_SOCKET%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Listening to a notification socket is supported (see section fd_notifiy for details on which messages are understood). NotifyAccess= is not fully supported though.</td>
</tr>
</table>

## systemd.exec

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#WorkingDirectory=">WorkingDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WorkingDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RootDirectory=">RootDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RootDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RootImage=">RootImage=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RootImage%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#MountAPIVFS=">MountAPIVFS=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MountAPIVFS%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#BindPaths=">BindPaths=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BindPaths%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#BindReadOnlyPaths=">BindReadOnlyPaths=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BindReadOnlyPaths%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#User=">User=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27User%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>The user id can be set for starting services. Currently only done for the main executable</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#Group=">Group=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27Group%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>The group id can be set for starting services. Currently only done for the main executable</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#DynamicUser=">DynamicUser=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DynamicUser%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SupplementaryGroups=">SupplementaryGroups=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27SupplementaryGroups%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>The supplementary group ids can be set for starting services. Currently only done for the main executable</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#PAMName=">PAMName=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PAMName%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#CapabilityBoundingSet=">CapabilityBoundingSet=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CapabilityBoundingSet%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#AmbientCapabilities=">AmbientCapabilities=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AmbientCapabilities%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#NoNewPrivileges=">NoNewPrivileges=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27NoNewPrivileges%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SecureBits=">SecureBits=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SecureBits%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SELinuxContext=">SELinuxContext=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SELinuxContext%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#AppArmorProfile=">AppArmorProfile=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AppArmorProfile%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SmackProcessLabel=">SmackProcessLabel=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SmackProcessLabel%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitCPU=">LimitCPU=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitCPU%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitFSIZE=">LimitFSIZE=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitFSIZE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitDATA=">LimitDATA=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitDATA%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitSTACK=">LimitSTACK=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitSTACK%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitCORE=">LimitCORE=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitCORE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitRSS=">LimitRSS=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitRSS%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitNOFILE=">LimitNOFILE=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitNOFILE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitAS=">LimitAS=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitAS%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitNPROC=">LimitNPROC=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitNPROC%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitMEMLOCK=">LimitMEMLOCK=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitMEMLOCK%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitLOCKS=">LimitLOCKS=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitLOCKS%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitSIGPENDING=">LimitSIGPENDING=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitSIGPENDING%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitMSGQUEUE=">LimitMSGQUEUE=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitMSGQUEUE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitNICE=">LimitNICE=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitNICE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitRTPRIO=">LimitRTPRIO=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitRTPRIO%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LimitRTTIME=">LimitRTTIME=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LimitRTTIME%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#UMask=">UMask=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27UMask%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#KeyringMode=">KeyringMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27KeyringMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#OOMScoreAdjust=">OOMScoreAdjust=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OOMScoreAdjust%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#TimerSlackNSec=">TimerSlackNSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TimerSlackNSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#Personality=">Personality=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Personality%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#IgnoreSIGPIPE=">IgnoreSIGPIPE=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IgnoreSIGPIPE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#Nice=">Nice=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Nice%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#CPUSchedulingPolicy=">CPUSchedulingPolicy=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUSchedulingPolicy%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#CPUSchedulingPriority=">CPUSchedulingPriority=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUSchedulingPriority%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#CPUSchedulingResetOnFork=">CPUSchedulingResetOnFork=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUSchedulingResetOnFork%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#CPUAffinity=">CPUAffinity=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUAffinity%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#NUMAPolicy=">NUMAPolicy=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27NUMAPolicy%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#NUMAMask=">NUMAMask=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27NUMAMask%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#IOSchedulingClass=">IOSchedulingClass=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IOSchedulingClass%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#IOSchedulingPriority=">IOSchedulingPriority=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IOSchedulingPriority%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ProtectSystem=">ProtectSystem=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ProtectSystem%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ProtectHome=">ProtectHome=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ProtectHome%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RuntimeDirectory=">RuntimeDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RuntimeDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#StateDirectory=">StateDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StateDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#CacheDirectory=">CacheDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CacheDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LogsDirectory=">LogsDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LogsDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ConfigurationDirectory=">ConfigurationDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConfigurationDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RuntimeDirectoryMode=">RuntimeDirectoryMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RuntimeDirectoryMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#StateDirectoryMode=">StateDirectoryMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StateDirectoryMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#CacheDirectoryMode=">CacheDirectoryMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CacheDirectoryMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LogsDirectoryMode=">LogsDirectoryMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LogsDirectoryMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ConfigurationDirectoryMode=">ConfigurationDirectoryMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConfigurationDirectoryMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RuntimeDirectoryPreserve=">RuntimeDirectoryPreserve=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RuntimeDirectoryPreserve%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#TimeoutCleanSec=">TimeoutCleanSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TimeoutCleanSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ReadWritePaths=">ReadWritePaths=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ReadWritePaths%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ReadOnlyPaths=">ReadOnlyPaths=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ReadOnlyPaths%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#InaccessiblePaths=">InaccessiblePaths=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27InaccessiblePaths%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#TemporaryFileSystem=">TemporaryFileSystem=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TemporaryFileSystem%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#PrivateTmp=">PrivateTmp=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PrivateTmp%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#PrivateDevices=">PrivateDevices=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PrivateDevices%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#PrivateNetwork=">PrivateNetwork=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PrivateNetwork%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#NetworkNamespacePath=">NetworkNamespacePath=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27NetworkNamespacePath%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#PrivateUsers=">PrivateUsers=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PrivateUsers%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ProtectHostname=">ProtectHostname=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ProtectHostname%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ProtectKernelTunables=">ProtectKernelTunables=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ProtectKernelTunables%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ProtectKernelModules=">ProtectKernelModules=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ProtectKernelModules%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ProtectKernelLogs=">ProtectKernelLogs=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ProtectKernelLogs%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#ProtectControlGroups=">ProtectControlGroups=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ProtectControlGroups%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RestrictAddressFamilies=">RestrictAddressFamilies=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RestrictAddressFamilies%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RestrictNamespaces=">RestrictNamespaces=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RestrictNamespaces%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LockPersonality=">LockPersonality=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LockPersonality%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#MemoryDenyWriteExecute=">MemoryDenyWriteExecute=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MemoryDenyWriteExecute%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RestrictRealtime=">RestrictRealtime=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RestrictRealtime%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RestrictSUIDSGID=">RestrictSUIDSGID=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RestrictSUIDSGID%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#RemoveIPC=">RemoveIPC=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RemoveIPC%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#PrivateMounts=">PrivateMounts=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PrivateMounts%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#MountFlags=">MountFlags=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MountFlags%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SystemCallFilter=">SystemCallFilter=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SystemCallFilter%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SystemCallErrorNumber=">SystemCallErrorNumber=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SystemCallErrorNumber%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SystemCallArchitectures=">SystemCallArchitectures=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SystemCallArchitectures%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#Environment=">Environment=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Environment%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#EnvironmentFile=">EnvironmentFile=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27EnvironmentFile%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#PassEnvironment=">PassEnvironment=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PassEnvironment%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#UnsetEnvironment=">UnsetEnvironment=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27UnsetEnvironment%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#StandardInput=">StandardInput=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StandardInput%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#StandardOutput=">StandardOutput=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StandardOutput%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#StandardError=">StandardError=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StandardError%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#StandardInputText=">StandardInputText=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StandardInputText%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#StandardInputData=">StandardInputData=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StandardInputData%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LogLevelMax=">LogLevelMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LogLevelMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LogExtraFields=">LogExtraFields=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LogExtraFields%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LogRateLimitIntervalSec=">LogRateLimitIntervalSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LogRateLimitIntervalSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#LogRateLimitBurst=">LogRateLimitBurst=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LogRateLimitBurst%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SyslogIdentifier=">SyslogIdentifier=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SyslogIdentifier%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SyslogFacility=">SyslogFacility=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SyslogFacility%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SyslogLevel=">SyslogLevel=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SyslogLevel%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#SyslogLevelPrefix=">SyslogLevelPrefix=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SyslogLevelPrefix%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#TTYPath=">TTYPath=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TTYPath%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#TTYReset=">TTYReset=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TTYReset%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#TTYVHangup=">TTYVHangup=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TTYVHangup%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#TTYVTDisallocate=">TTYVTDisallocate=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TTYVTDisallocate%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#UtmpIdentifier=">UtmpIdentifier=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27UtmpIdentifier%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#UtmpMode=">UtmpMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27UtmpMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$PATH">$PATH</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PATH%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$LANG">$LANG</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LANG%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$USER">$USER</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27USER%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$LOGNAME">$LOGNAME</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LOGNAME%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$HOME">$HOME</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27HOME%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$SHELL">$SHELL</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SHELL%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$INVOCATION_ID">$INVOCATION_ID</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27INVOCATION_ID%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$XDG_RUNTIME_DIR">$XDG_RUNTIME_DIR</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27XDG_RUNTIME_DIR%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$RUNTIME_DIRECTORY">$RUNTIME_DIRECTORY</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RUNTIME_DIRECTORY%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$STATE_DIRECTORY">$STATE_DIRECTORY</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27STATE_DIRECTORY%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$CACHE_DIRECTORY">$CACHE_DIRECTORY</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CACHE_DIRECTORY%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$LOGS_DIRECTORY">$LOGS_DIRECTORY</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27LOGS_DIRECTORY%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$CONFIGURATION_DIRECTORY">$CONFIGURATION_DIRECTORY</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CONFIGURATION_DIRECTORY%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$MAINPID">$MAINPID</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MAINPID%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$MANAGERPID">$MANAGERPID</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MANAGERPID%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$LISTEN_FDS">$LISTEN_FDS</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27LISTEN_FDS%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Providing number of filedescriptors is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$LISTEN_PID">$LISTEN_PID</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27LISTEN_PID%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Provifing the listen_pid to the child is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$LISTEN_FDNAMES">$LISTEN_FDNAMES</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27LISTEN_FDNAMES%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Providing names for filedescriptors is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$NOTIFY_SOCKET">$NOTIFY_SOCKET</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27NOTIFY_SOCKET%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Listening to a notification socket is supported (see section fd_notifiy for details on which messages are understood). NotifyAccess= is not fully supported though.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$WATCHDOG_PID">$WATCHDOG_PID</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WATCHDOG_PID%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$WATCHDOG_USEC">$WATCHDOG_USEC</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WATCHDOG_USEC%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$TERM">$TERM</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TERM%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$JOURNAL_STREAM">$JOURNAL_STREAM</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27JOURNAL_STREAM%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$SERVICE_RESULT">$SERVICE_RESULT</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SERVICE_RESULT%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$EXIT_CODE">$EXIT_CODE</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27EXIT_CODE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$EXIT_STATUS">$EXIT_STATUS</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27EXIT_STATUS%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.exec.html#$PIDFILE">$PIDFILE</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PIDFILE%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
</table>

## systemd.kill

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.kill.html#KillMode=">KillMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27KillMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.kill.html#KillSignal=">KillSignal=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27KillSignal%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.kill.html#RestartKillSignal=">RestartKillSignal=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RestartKillSignal%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.kill.html#SendSIGHUP=">SendSIGHUP=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SendSIGHUP%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.kill.html#SendSIGKILL=">SendSIGKILL=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SendSIGKILL%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.kill.html#FinalKillSignal=">FinalKillSignal=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27FinalKillSignal%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.kill.html#WatchdogSignal=">WatchdogSignal=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WatchdogSignal%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
</table>

## systemd.path

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.path.html#PathExists=">PathExists=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PathExists%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.path.html#PathExistsGlob=">PathExistsGlob=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PathExistsGlob%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.path.html#PathChanged=">PathChanged=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PathChanged%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.path.html#PathModified=">PathModified=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PathModified%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.path.html#DirectoryNotEmpty=">DirectoryNotEmpty=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DirectoryNotEmpty%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.path.html#Unit=">Unit=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27Unit%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.path.html#MakeDirectory=">MakeDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MakeDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.path.html#DirectoryMode=">DirectoryMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DirectoryMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
</table>

## systemd.resource-control

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#CPU">CPU</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPU%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#Memory">Memory</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Memory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IO">IO</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IO%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#CPUAccounting=">CPUAccounting=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUAccounting%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#CPUWeight=">CPUWeight=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUWeight%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#StartupCPUWeight=">StartupCPUWeight=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StartupCPUWeight%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#CPUQuota=">CPUQuota=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUQuota%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#CPUQuotaPeriodSec=">CPUQuotaPeriodSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUQuotaPeriodSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#AllowedCPUs=">AllowedCPUs=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AllowedCPUs%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#AllowedMemoryNodes=">AllowedMemoryNodes=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AllowedMemoryNodes%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#MemoryAccounting=">MemoryAccounting=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MemoryAccounting%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#MemoryMin=">MemoryMin=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MemoryMin%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#MemoryLow=">MemoryLow=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MemoryLow%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#MemoryHigh=">MemoryHigh=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MemoryHigh%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#MemoryMax=">MemoryMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MemoryMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#MemorySwapMax=">MemorySwapMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MemorySwapMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#TasksAccounting=">TasksAccounting=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TasksAccounting%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#TasksMax=">TasksMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TasksMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IOAccounting=">IOAccounting=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IOAccounting%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IOWeight=">IOWeight=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IOWeight%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#StartupIOWeight=">StartupIOWeight=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StartupIOWeight%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IODeviceWeight=">IODeviceWeight=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IODeviceWeight%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IOReadBandwidthMax=">IOReadBandwidthMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IOReadBandwidthMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IOWriteBandwidthMax=">IOWriteBandwidthMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IOWriteBandwidthMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IOReadIOPSMax=">IOReadIOPSMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IOReadIOPSMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IOWriteIOPSMax=">IOWriteIOPSMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IOWriteIOPSMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IODeviceLatencyTargetSec=">IODeviceLatencyTargetSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IODeviceLatencyTargetSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IPAccounting=">IPAccounting=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IPAccounting%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IPAddressAllow=">IPAddressAllow=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IPAddressAllow%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IPAddressDeny=">IPAddressDeny=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IPAddressDeny%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IPIngressFilterPath=">IPIngressFilterPath=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IPIngressFilterPath%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#IPEgressFilterPath=">IPEgressFilterPath=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IPEgressFilterPath%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#DeviceAllow=">DeviceAllow=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DeviceAllow%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#DevicePolicy=auto|closed|strict">DevicePolicy=auto|closed|strict</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DevicePolicy%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#Slice=">Slice=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Slice%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#Delegate=">Delegate=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Delegate%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#DisableControllers=">DisableControllers=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DisableControllers%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#CPUShares=">CPUShares=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CPUShares%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#StartupCPUShares=">StartupCPUShares=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StartupCPUShares%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#MemoryLimit=">MemoryLimit=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MemoryLimit%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#BlockIOAccounting=">BlockIOAccounting=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BlockIOAccounting%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#BlockIOWeight=">BlockIOWeight=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BlockIOWeight%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#StartupBlockIOWeight=">StartupBlockIOWeight=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StartupBlockIOWeight%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#BlockIODeviceWeight=">BlockIODeviceWeight=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BlockIODeviceWeight%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#BlockIOReadBandwidth=">BlockIOReadBandwidth=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BlockIOReadBandwidth%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html#BlockIOWriteBandwidth=">BlockIOWriteBandwidth=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BlockIOWriteBandwidth%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
</table>

## systemd.service

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#Type=">Type=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27Type%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Types are partly supported. Simple, dbus, notify, oneshot are supported. Forking, idle are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#RemainAfterExit=">RemainAfterExit=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RemainAfterExit%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#GuessMainPID=">GuessMainPID=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27GuessMainPID%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#PIDFile=">PIDFile=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PIDFile%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#BusName=">BusName=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27BusName%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Setting a bus name to wait for services of type dbus is supported.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#ExecStart=">ExecStart=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27ExecStart%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Exec'ing the command given is supported. The return value is checked for oneshot services. Ignoring the return value with the '-' prefix is supported, other prefixes are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#ExecStartPre=">ExecStartPre=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27ExecStartPre%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#ExecStartPost=">ExecStartPost=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27ExecStartPost%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#ExecCondition=">ExecCondition=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ExecCondition%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#ExecReload=">ExecReload=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ExecReload%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#ExecStop=">ExecStop=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27ExecStop%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#ExecStopPost=">ExecStopPost=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27ExecStopPost%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#RestartSec=">RestartSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RestartSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#TimeoutStartSec=">TimeoutStartSec=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27TimeoutStartSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>The time a services needs to start can be limited</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#TimeoutStopSec=">TimeoutStopSec=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27TimeoutStopSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>The time a services needs to stop can be limited</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#TimeoutAbortSec=">TimeoutAbortSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TimeoutAbortSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#TimeoutSec=">TimeoutSec=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27TimeoutSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>The time a services needs to start/stop can be limited</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#RuntimeMaxSec=">RuntimeMaxSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RuntimeMaxSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#WatchdogSec=">WatchdogSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WatchdogSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#Restart=">Restart=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27Restart%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Restart is partially supported. The settings 'always' and 'no' are supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#SuccessExitStatus=">SuccessExitStatus=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SuccessExitStatus%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#RestartPreventExitStatus=">RestartPreventExitStatus=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RestartPreventExitStatus%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#RestartForceExitStatus=">RestartForceExitStatus=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RestartForceExitStatus%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#RootDirectoryStartOnly=">RootDirectoryStartOnly=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RootDirectoryStartOnly%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#NonBlocking=">NonBlocking=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27NonBlocking%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#NotifyAccess=">NotifyAccess=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27NotifyAccess%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Not fully supported. All settings are accepted but are not being enforced right now. Acts as if 'all' was set.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#Sockets=">Sockets=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27Sockets%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Adding more socket files to servcies is supported. But only so that one socket belongs to only one service (sytsemd allows for sockets to belong to multiple services).</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#FileDescriptorStoreMax=">FileDescriptorStoreMax=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27FileDescriptorStoreMax%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#USBFunctionDescriptors=">USBFunctionDescriptors=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27USBFunctionDescriptors%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#USBFunctionStrings=">USBFunctionStrings=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27USBFunctionStrings%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.service.html#OOMPolicy=">OOMPolicy=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OOMPolicy%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
</table>

## systemd.socket

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ListenStream=">ListenStream=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27ListenStream%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Opening streaming sockets is supported. The whole IPv4 and IPv6 stuff needs some attention though</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ListenDatagram=">ListenDatagram=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27ListenDatagram%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Opening datagram sockets is supported. The whole IPv4 and IPv6 stuff needs some attention though</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ListenSequentialPacket=">ListenSequentialPacket=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27ListenSequentialPacket%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Opening sequential packet sockets is supported.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ListenFIFO=">ListenFIFO=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27ListenFIFO%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Opening FIFOs is supported. Filemode setting is not supported as of yet though.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ListenSpecial=">ListenSpecial=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ListenSpecial%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ListenNetlink=">ListenNetlink=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ListenNetlink%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ListenMessageQueue=">ListenMessageQueue=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ListenMessageQueue%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ListenUSBFunction=">ListenUSBFunction=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ListenUSBFunction%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SocketProtocol=">SocketProtocol=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SocketProtocol%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#BindIPv6Only=">BindIPv6Only=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BindIPv6Only%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Backlog=">Backlog=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Backlog%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#BindToDevice=">BindToDevice=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BindToDevice%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SocketUser=">SocketUser=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SocketUser%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SocketGroup=">SocketGroup=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SocketGroup%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SocketMode=">SocketMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SocketMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#DirectoryMode=">DirectoryMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DirectoryMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Accept=">Accept=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27Accept%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Only the setting 'no' is supported. Inted-style activation is not yet supported.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Writable=">Writable=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Writable%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#MaxConnections=">MaxConnections=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MaxConnections%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#MaxConnectionsPerSource=">MaxConnectionsPerSource=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MaxConnectionsPerSource%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#KeepAlive=">KeepAlive=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27KeepAlive%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#KeepAliveTimeSec=">KeepAliveTimeSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27KeepAliveTimeSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#KeepAliveIntervalSec=">KeepAliveIntervalSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27KeepAliveIntervalSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#KeepAliveProbes=">KeepAliveProbes=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27KeepAliveProbes%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#NoDelay=">NoDelay=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27NoDelay%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Priority=">Priority=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Priority%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#DeferAcceptSec=">DeferAcceptSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DeferAcceptSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ReceiveBuffer=">ReceiveBuffer=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ReceiveBuffer%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SendBuffer=">SendBuffer=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SendBuffer%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#IPTOS=">IPTOS=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IPTOS%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#IPTTL=">IPTTL=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IPTTL%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Mark=">Mark=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Mark%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ReusePort=">ReusePort=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ReusePort%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SmackLabel=">SmackLabel=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SmackLabel%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SmackLabelIPIn=">SmackLabelIPIn=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SmackLabelIPIn%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SmackLabelIPOut=">SmackLabelIPOut=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SmackLabelIPOut%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#SELinuxContextFromNet=">SELinuxContextFromNet=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SELinuxContextFromNet%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#PipeSize=">PipeSize=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PipeSize%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#MessageQueueMaxMessages=">MessageQueueMaxMessages=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27MessageQueueMaxMessages%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#FreeBind=">FreeBind=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27FreeBind%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Transparent=">Transparent=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Transparent%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Broadcast=">Broadcast=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Broadcast%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#PassCredentials=">PassCredentials=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PassCredentials%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#PassSecurity=">PassSecurity=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PassSecurity%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#TCPCongestion=">TCPCongestion=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TCPCongestion%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ExecStartPre=">ExecStartPre=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27ExecStartPre%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ExecStartPost=">ExecStartPost=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27ExecStartPost%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ExecStopPre=">ExecStopPre=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ExecStopPre%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#ExecStopPost=">ExecStopPost=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27ExecStopPost%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#TimeoutSec=">TimeoutSec=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27TimeoutSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>The time a services needs to start/stop can be limited</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Service=">Service=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27Service%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Adding a socket explicitly to a service is supported.</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#RemoveOnStop=">RemoveOnStop=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RemoveOnStop%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#Symlinks=">Symlinks=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Symlinks%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#FileDescriptorName=">FileDescriptorName=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27FileDescriptorName%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Naming the sockets for passing in $LISTEN_FDNAMES is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#TriggerLimitIntervalSec=">TriggerLimitIntervalSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TriggerLimitIntervalSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.socket.html#TriggerLimitBurst=">TriggerLimitBurst=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27TriggerLimitBurst%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
</table>

## systemd.timer

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#OnActiveSec=">OnActiveSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnActiveSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#OnBootSec=">OnBootSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnBootSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#OnStartupSec=">OnStartupSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnStartupSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#OnUnitActiveSec=">OnUnitActiveSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnUnitActiveSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#OnUnitInactiveSec=">OnUnitInactiveSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnUnitInactiveSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#OnCalendar=">OnCalendar=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnCalendar%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#AccuracySec=">AccuracySec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AccuracySec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#RandomizedDelaySec=">RandomizedDelaySec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RandomizedDelaySec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#OnClockChange=">OnClockChange=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnClockChange%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#OnTimezoneChange=">OnTimezoneChange=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnTimezoneChange%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#Unit=">Unit=</a></td>
  <td>❓</td>
  <td><a href="https://github.com/search?q=%27Unit%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#Persistent=">Persistent=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Persistent%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#WakeSystem=">WakeSystem=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27WakeSystem%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.timer.html#RemainAfterElapse=">RemainAfterElapse=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RemainAfterElapse%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
</table>

## systemd.unit

<table>
  <tr>
    <th>Term</th>
    <th>Supported</th>
    <th>Search</th>
    <th>Notes</th>
  </tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Description=">Description=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27Description%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Descriptions are read and will be displayed by the control interface</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Documentation=">Documentation=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Documentation%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Wants=">Wants=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27Wants%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Specifying which units to pull in is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Requires=">Requires=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27Requires%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Specifying which units to pull in is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Requisite=">Requisite=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Requisite%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#BindsTo=">BindsTo=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27BindsTo%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#PartOf=">PartOf=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PartOf%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Conflicts=">Conflicts=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Conflicts%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Before=">Before=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27Before%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Ordering of units according to before/after relation is supported fully</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#After=">After=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27After%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Ordering of units according to before/after relation is supported fully</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#OnFailure=">OnFailure=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnFailure%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#PropagatesReloadTo=">PropagatesReloadTo=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27PropagatesReloadTo%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ReloadPropagatedFrom=">ReloadPropagatedFrom=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ReloadPropagatedFrom%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#JoinsNamespaceOf=">JoinsNamespaceOf=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27JoinsNamespaceOf%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#RequiresMountsFor=">RequiresMountsFor=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RequiresMountsFor%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#OnFailureJobMode=">OnFailureJobMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27OnFailureJobMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#IgnoreOnIsolate=">IgnoreOnIsolate=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27IgnoreOnIsolate%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#StopWhenUnneeded=">StopWhenUnneeded=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StopWhenUnneeded%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#RefuseManualStart=">RefuseManualStart=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RefuseManualStart%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#RefuseManualStop=">RefuseManualStop=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RefuseManualStop%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AllowIsolate=">AllowIsolate=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AllowIsolate%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#DefaultDependencies=">DefaultDependencies=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DefaultDependencies%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#CollectMode=">CollectMode=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27CollectMode%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#FailureAction=">FailureAction=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27FailureAction%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#SuccessAction=">SuccessAction=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SuccessAction%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#FailureActionExitStatus=">FailureActionExitStatus=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27FailureActionExitStatus%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#SuccessActionExitStatus=">SuccessActionExitStatus=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SuccessActionExitStatus%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#JobTimeoutSec=">JobTimeoutSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27JobTimeoutSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#JobRunningTimeoutSec=">JobRunningTimeoutSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27JobRunningTimeoutSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#JobTimeoutAction=">JobTimeoutAction=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27JobTimeoutAction%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#JobTimeoutRebootArgument=">JobTimeoutRebootArgument=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27JobTimeoutRebootArgument%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#StartLimitIntervalSec=">StartLimitIntervalSec=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StartLimitIntervalSec%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#StartLimitBurst=">StartLimitBurst=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StartLimitBurst%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#StartLimitAction=">StartLimitAction=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27StartLimitAction%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#RebootArgument=">RebootArgument=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27RebootArgument%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#SourcePath=">SourcePath=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27SourcePath%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionArchitecture=">ConditionArchitecture=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionArchitecture%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionVirtualization=">ConditionVirtualization=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionVirtualization%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionHost=">ConditionHost=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionHost%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionKernelCommandLine=">ConditionKernelCommandLine=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionKernelCommandLine%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionKernelVersion=">ConditionKernelVersion=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionKernelVersion%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionSecurity=">ConditionSecurity=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionSecurity%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionCapability=">ConditionCapability=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionCapability%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionACPower=">ConditionACPower=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionACPower%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionNeedsUpdate=">ConditionNeedsUpdate=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionNeedsUpdate%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionFirstBoot=">ConditionFirstBoot=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionFirstBoot%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionPathExists=">ConditionPathExists=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionPathExists%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionPathExistsGlob=">ConditionPathExistsGlob=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionPathExistsGlob%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionPathIsDirectory=">ConditionPathIsDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionPathIsDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionPathIsSymbolicLink=">ConditionPathIsSymbolicLink=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionPathIsSymbolicLink%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionPathIsMountPoint=">ConditionPathIsMountPoint=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionPathIsMountPoint%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionPathIsReadWrite=">ConditionPathIsReadWrite=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionPathIsReadWrite%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionDirectoryNotEmpty=">ConditionDirectoryNotEmpty=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionDirectoryNotEmpty%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionFileNotEmpty=">ConditionFileNotEmpty=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionFileNotEmpty%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionFileIsExecutable=">ConditionFileIsExecutable=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionFileIsExecutable%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionUser=">ConditionUser=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionUser%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionGroup=">ConditionGroup=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionGroup%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionControlGroupController=">ConditionControlGroupController=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionControlGroupController%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionMemory=">ConditionMemory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionMemory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#ConditionCPUs=">ConditionCPUs=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27ConditionCPUs%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertArchitecture=">AssertArchitecture=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertArchitecture%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertVirtualization=">AssertVirtualization=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertVirtualization%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertHost=">AssertHost=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertHost%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertKernelCommandLine=">AssertKernelCommandLine=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertKernelCommandLine%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertKernelVersion=">AssertKernelVersion=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertKernelVersion%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertSecurity=">AssertSecurity=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertSecurity%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertCapability=">AssertCapability=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertCapability%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertACPower=">AssertACPower=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertACPower%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertNeedsUpdate=">AssertNeedsUpdate=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertNeedsUpdate%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertFirstBoot=">AssertFirstBoot=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertFirstBoot%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertPathExists=">AssertPathExists=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertPathExists%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertPathExistsGlob=">AssertPathExistsGlob=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertPathExistsGlob%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertPathIsDirectory=">AssertPathIsDirectory=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertPathIsDirectory%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertPathIsSymbolicLink=">AssertPathIsSymbolicLink=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertPathIsSymbolicLink%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertPathIsMountPoint=">AssertPathIsMountPoint=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertPathIsMountPoint%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertPathIsReadWrite=">AssertPathIsReadWrite=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertPathIsReadWrite%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertDirectoryNotEmpty=">AssertDirectoryNotEmpty=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertDirectoryNotEmpty%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertFileNotEmpty=">AssertFileNotEmpty=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertFileNotEmpty%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertFileIsExecutable=">AssertFileIsExecutable=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertFileIsExecutable%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertUser=">AssertUser=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertUser%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertGroup=">AssertGroup=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertGroup%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#AssertControlGroupController=">AssertControlGroupController=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27AssertControlGroupController%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Alias=">Alias=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Alias%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#WantedBy=">WantedBy=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27WantedBy%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Specifying which units pull this unit in is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#RequiredBy=">RequiredBy=</a></td>
  <td>✔️</td>
  <td><a href="https://github.com/search?q=%27RequiredBy%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td>Specifying which units pull this unit in is supported</td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#Also=">Also=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27Also%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
<tr>
  <td><a href="https://www.freedesktop.org/software/systemd/man/systemd.unit.html#DefaultInstance=">DefaultInstance=</a></td>
  <td>❌</td>
  <td><a href="https://github.com/search?q=%27DefaultInstance%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
  <td></td>
</tr>
</table>
