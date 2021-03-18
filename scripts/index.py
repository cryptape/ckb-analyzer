#!/usr/bin/env python

from __future__ import print_function
import json
import sys
from os import listdir, path

DASHBOARDS_DIR = path.join(path.dirname(__file__), '../dashboards')
INDEX_FILE_PATH = path.join(path.dirname(__file__), '../dashboards.md')

indexes = []
for filename in listdir(DASHBOARDS_DIR):
    filepath = path.join(DASHBOARDS_DIR, filename)
    if not path.isfile(filepath) or not filepath.endswith('.json'):
        continue

    # Ignore Home Dashboard
    if 'Home.json' in filename:
        continue

    with open(filepath) as fp:
        dashboard = json.load(fp)
        title = dashboard['title']
        panels = dashboard['panels']
        notes = None
        for panel in panels:
            if panel['title'] == 'Notes' and panel['type'] == 'text':
                notes = panel['options']['content']
                break

        if notes is None:
            print('{} {} does not have "Notes" panel'.format(filepath, title), file=sys.stderr)

        indexes.append((title, filepath, notes))

with open(INDEX_FILE_PATH, 'w') as fp:
    print('# Dashboards', file=fp)
    for (title, filepath, notes) in indexes:
        filepath = filepath.replace(' ', '%20')
        print('## [{}]({})'.format(title, filepath), file=fp)
        print('', file=fp)
        print('{}'.format(notes), file=fp)
        print('', file=fp)
