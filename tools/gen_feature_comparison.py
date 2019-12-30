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
            out.write(dedent("""\
                <tr>
                  <td><a href="https://www.freedesktop.org/software/systemd/man/{page}.html#{term}">{term}</a></td>
                  <td>{icon}</td>
                  <td><a href="https://github.com/search?q=%27{term_noeq}%27+repo%3AKillingSpark%2Frustysd+language%3ARust&type=Code">Search</a></td>
                  <td></td>
                </tr>
                """).format(
                    term=term, term_noeq=term_noeq, page=page, icon=icon))
            
        out.write("</table>\n")

    out.close()


if __name__ == '__main__':
    main()
