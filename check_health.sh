#!/bin/sh

pgrep -f puzzles_crosswise && pgrep -f "nginx: master" && pgrep -f "nginx: worker"
