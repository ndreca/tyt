#!/bin/sh
# PostToolUse hook: after a Bash call that renders, copies the slot's new PNGs
# and model files into _runs/<run>/passes/<n>.
input=$(cat)
case $input in *"object render"*) ;; *) exit 0 ;; esac
slot=${CLAUDE_PROJECT_DIR:-$PWD}
out=$(dirname "$slot")/_runs/$(basename "$slot")
[ -f "$out/snapshot-marker" ] || exit 0
pngs=$(cd "$slot" && find . -path ./.claude -prune -o -name '*.png' -newer "$out/snapshot-marker" -print)
[ -n "$pngs" ] || exit 0
n=$(ls -d "$out"/passes/[0-9]* 2> /dev/null | wc -l | tr -d ' ')
pass=$out/passes/$(printf '%02d' $((n + 1)))
mkdir -p "$pass"
(cd "$slot" && printf '%s\n' "$pngs" | while read -r f; do cp "$f" "$pass/"; done)
(cd "$slot" && find . -path ./.claude -prune -o -name '*.ts' -print | while read -r f; do cp "$f" "$pass/"; done)
touch "$out/snapshot-marker"
exit 0
