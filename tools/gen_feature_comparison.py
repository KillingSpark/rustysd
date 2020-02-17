#!/usr/bin/python
# coding: utf-8
from __future__ import unicode_literals

import argparse
import codecs
import subprocess
import sys
from xml.etree import ElementTree
from textwrap import dedent


RELEVANT_PAGES = [
    'sd_notify',
    'systemd.exec',
    'systemd.kill',
    'systemd.path',
    'systemd.resource-control',
    'systemd.service',
    'systemd.socket',
    'systemd.timer',
    'systemd.unit'
]

ICON_TICK = "✔️"
ICON_CROSS = "❌"
ICON_QMARK = "❓"

# tracks which features have actually been inserted to find cases where names change / features get removed in the future.
USED_FEATURES = {}

SUPPORTED_FEATURES = {
    "READY": {"icon": ICON_TICK, "text": "Waiting for ready notification for service-type notify is supported"},
    "STATUS": {"icon": ICON_TICK, "text": "Sending free-text status updates to be displayed for the user is supported"},
    "NOTIFY_SOCKET": {"icon": ICON_TICK, "text": "Listening to a notification socket is supported (see section fd_notifiy for details on which messages are understood). NotifyAccess= is not fully supported though."},
    "LISTEN_FDS": {"icon": ICON_TICK, "text": "Providing number of filedescriptors is supported"},
    "LISTEN_FDNAMES": {"icon": ICON_TICK, "text": "Providing names for filedescriptors is supported"},
    "LISTEN_PID": {"icon": ICON_TICK, "text": "Provifing the listen_pid to the child is supported"},
    "After": {"icon": ICON_TICK, "text":  "Ordering of units according to before/after relation is supported fully"},
    "Before": {"icon": ICON_TICK, "text": "Ordering of units according to before/after relation is supported fully"},
    "Type": {"icon": ICON_QMARK, "text": "Types are partly supported. Simple, dbus, notify, oneshot are supported. Forking, idle are not."},
    "Restart": {"icon": ICON_QMARK, "text": "Restart is partially supported. The settings 'always' and 'no' are supported"},
    "BusName": {"icon": ICON_TICK, "text": "Setting a bus name to wait for services of type dbus is supported."},
    "NotifyAccess": {"icon": ICON_QMARK, "text": "Not fully supported. All settings are accepted but are not being enforced right now. Acts as if 'all' was set."},
    "Sockets": {"icon": ICON_QMARK, "text": "Adding more socket files to servcies is supported. But only so that one socket belongs to only one service (sytsemd allows for sockets to belong to multiple services)."},
    "ListenStream": {"icon": ICON_TICK, "text": "Opening streaming sockets is supported. The whole IPv4 and IPv6 stuff needs some attention though"},
    "ListenDatagram": {"icon": ICON_TICK, "text": "Opening datagram sockets is supported. The whole IPv4 and IPv6 stuff needs some attention though"},
    "ListenSequentialPacket": {"icon": ICON_TICK, "text": "Opening sequential packet sockets is supported."},
    "ListenFIFO": {"icon": ICON_TICK, "text": "Opening FIFOs is supported. Filemode setting is not supported as of yet though."},
    "Accept": {"icon": ICON_QMARK, "text": "Only the setting 'no' is supported. Inted-style activation is not yet supported."},
    "ExecStart": {"icon": ICON_TICK, "text": "Exec'ing the command given is supported. The return value is checked for oneshot services. Ignoring the return value with the '-' prefix is supported, other prefixes are not."},
    "ExecStartPre": {"icon": ICON_QMARK,  "text": "Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not."},
    "ExecStartPost": {"icon": ICON_QMARK, "text": "Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not."},
    "ExecStop": {"icon": ICON_QMARK,      "text": "Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not."},
    "ExecStopPost": {"icon": ICON_QMARK,  "text": "Allowing commands to be run is supported. The return value is checked. Ignoring the return value with the '-' prefix is supported, other prefixes are not."},
    "Service": {"icon": ICON_TICK, "text": "Adding a socket explicitly to a service is supported."},
    "FileDescriptorName": {"icon": ICON_TICK, "text": "Naming the sockets for passing in $LISTEN_FDNAMES is supported"},
    "Description": {"icon": ICON_TICK, "text": "Descriptions are read and will be displayed by the control interface"},
    "Wants": {"icon": ICON_TICK, "text": "Specifying which units to pull in is supported"},
    "Requires": {"icon": ICON_TICK, "text": "Specifying which units to pull in is supported"},
    "WantedBy": {"icon": ICON_TICK,   "text": "Specifying which units pull this unit in is supported"},
    "RequiredBy": {"icon": ICON_TICK, "text": "Specifying which units pull this unit in is supported"},
    "TimeoutStartSec": {"icon": ICON_TICK, "text": "The time a services needs to start can be limited"},
    "TimeoutStopSec": {"icon": ICON_TICK, "text": "The time a services needs to stop can be limited"},
    "TimeoutSec": {"icon": ICON_TICK, "text": "The time a services needs to start/stop can be limited"},
    "User": {"icon": ICON_QMARK, "text": "The user id can be set for starting services. Currently only done for the main executable"},
    "Group": {"icon": ICON_QMARK, "text": "The group id can be set for starting services. Currently only done for the main executable"},
    "SupplementaryGroups": {"icon": ICON_QMARK, "text": "The supplementary group ids can be set for starting services. Currently only done for the main executable"},
}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--systemd-repo", default="../systemd")
    args = parser.parse_args()

    out = codecs.open("feature-comparison.md", "w", "utf-8")

    out.write(dedent("""
    # Feature comparison
    This document is auto-generated. It pulls all features from the xml-doc from systemd and checks whether the features is supported
    in rustysd. (shoutout to [wmanley](https://github.com/wmanley) who wrote the initial script!). Note that this shows a lot 
    of crosses. This can have two reasons: 
    1. The most likely case is that the feature is not (and will likely never) be supported because it is out of scope of this project (see Readme on how that is determined)
    1. The feature is not yet supported but should be. If thats the case please file an issue and I will push it to the top of the priority list.

    This document is meant as a simple way of checking whether all features you need from systemd are supported in rustysd.
        """))
    for page in RELEVANT_PAGES:
        out.write(dedent("""
            ## {page}

            <table>
              <tr>
                <th>Term</th>
                <th>Supported</th>
                <th>Search</th>
                <th>Notes</th>
              </tr>
              """).format(page=page))
        tree = ElementTree.ElementTree()
        tree.parse("%s/man/%s.xml" % (args.systemd_repo, page))
        for elem in tree.findall('.//term'):
            elem2 = elem.findall('./varname')
            if elem2:
                elem = elem2[0]
            term = elem.text
            if not term:
                continue
            term_noeq = term.split("=")[0].replace("$", "")
            present = 0 == subprocess.call(
                ['git', 'grep', '-i', '-q', '"%s"' % term_noeq, 'src/*.rs'])
            icon = '❓' if present else '❌'
            text = ""

            if term_noeq in SUPPORTED_FEATURES:
                icon = SUPPORTED_FEATURES[term_noeq]["icon"]
                text = SUPPORTED_FEATURES[term_noeq]["text"]
                USED_FEATURES[term_noeq] = True
            
            else:
                if present:
                    print("[WARN] found feature in repo but not in supported features: " + term_noeq)

            out.write(dedent("""\
                <tr>
                  <td><a href="https://www.freedesktop.org/software/systemd/man/{page}.html#{term}">{term}</a></td>
                  <td>{icon}</td>
                  <td><a href="https://github.com/search?q=%27{term_noeq}%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
                  <td>{text}</td>
                </tr>
                """).format(
                    term=term, term_noeq=term_noeq, page=page, icon=icon, text=text))
            
        out.write("</table>\n")

    out.close()

def assert_all_features_used():
    for name in SUPPORTED_FEATURES:
        if not name in USED_FEATURES:
            print("Feature supported but not mentioned in doc: " + name)

if __name__ == '__main__':
    main()
    assert_all_features_used()