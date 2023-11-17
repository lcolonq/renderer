#!/bin/sh
cd "$(dirname "$0")" || exit;
(cd deps/OpenSeeFace/ || exit; steam-run poetry run python facetracker.py -c 1 >/dev/null &)
../terminal/st -T "Colonq's Face" -e ./run.sh
