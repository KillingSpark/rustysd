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

SUPPORTED_FEATURES = {
    "READY": {"icon": ICON_TICK, "text": "Waiting for ready notification for service-type notify is supported"},
    "STATUS": {"icon": ICON_TICK, "text": "Sending free-text status updates to be displayed for the user is supported"},
    "NOTIFY_SOCKET": {"icon": ICON_TICK, "text": "Listening to a notification socket is supported (see section fd_notifiy for details on which messages are understood). NotifyAccess= is not fully supported though."},
    "LISTEN_FDS": {"icon": ICON_TICK, "text": "Providing number of filedescriptors is supported"},
    "LISTEN_FDNAMES": {"icon": ICON_TICK, "text": "Providing names for filedescriptors is supported"},
    "LISTEN_PID": {"icon": ICON_TICK, "text": "Provifing the listen_pid to the child is supported"},
    "After": {"icon": ICON_TICK, "text":  "Ordering of units according to before/after relation is supported fully"},
    "Before": {"icon": ICON_TICK, "text": "Ordering of units according to before/after relation is supported fully"},
    "Type": {"icon": ICON_QMARK, "text": "Types are partly supported. Simple, dbus, notify are supported. Forking, oneshot, idle are not."},
    "Restart": {"icon": ICON_QMARK, "text": "Restart is partially supported. The settings 'always' and 'no' are supported"},
    "BusName": {"icon": ICON_TICK, "text": "Setting a bus name to wait for services of type dbus is supported."},
    "NotifyAccess": {"icon": ICON_QMARK, "text": "Not fully supported. All settings are accepted but are not being enforced right now. Acts as if 'all' was set."},
    "Sockets": {"icon": ICON_QMARK, "text": "Adding more socket files to servcies is supported. But only so that one socket belongs to only one service (sytsemd allows for sockets to belong to multiple services)."},
    "ListenStream": {"icon": ICON_TICK, "text": "Opening streaming sockets is supported. The whole IPv4 and IPv6 stuff needs some attention though"},
    "ListenDatagram": {"icon": ICON_TICK, "text": "Opening datagram sockets is supported. The whole IPv4 and IPv6 stuff needs some attention though"},
    "ListenSequentialPacket": {"icon": ICON_TICK, "text": "Opening sequential packet sockets is supported."},
    "ListenFIFO": {"icon": ICON_TICK, "text": "Opening FIFOs is supported. Filemode setting is not supported as of yet though."},
    "Accept": {"icon": ICON_QMARK, "text": "Only the setting 'no' is supported. Inted-style activation is not yet supported."},
    "Service": {"icon": ICON_TICK, "text": "Adding a socket explicitly to a service is supported."},
    "FileDescriptorName": {"icon": ICON_TICK, "text": "Naming the sockets for passing in $LISTEN_FDNAMES is supported"},
    "Description": {"icon": ICON_TICK, "text": "Descriptions are read and will be displayed by the control interface"},
    "Wants": {"icon": ICON_TICK, "text": "Specifying which units to pull in is supported"},
    "Requires": {"icon": ICON_TICK, "text": "Specifying which units to pull in is supported"},
    "WantedBy": {"icon": ICON_TICK,   "text": "Specifying which units pull this unit in is supported"},
    "RequiredBy": {"icon": ICON_TICK, "text": "Specifying which units pull this unit in is supported"},
}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--systemd-repo", default="../systemd")
    args = parser.parse_args()

    out = codecs.open("feature-comparison.md", "w", "utf-8")

    for page in RELEVANT_PAGES:
        out.write(dedent("""
            # {page}

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


if __name__ == '__main__':
    main()
