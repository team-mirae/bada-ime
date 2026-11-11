#!/bin/sh
# Installs the BadaIME.app GitHub Actions built (.github/workflows/mac.yml), without building here:
# the newest successful run on a branch (default: the current one), or a given run id.
#   mac/tools/install-ci.sh            the current branch
#   mac/tools/install-ci.sh main       a branch
#   mac/tools/install-ci.sh 1234567    a run id
# Needs the GitHub CLI (gh), signed in.
set -e
cd "$(dirname "$0")/.."
arg=${1:-$(git rev-parse --abbrev-ref HEAD)}
case $arg in
  *[!0-9]*) run=$(gh run list --workflow mac.yml --branch "$arg" --status success -L 1 --json databaseId -q '.[0].databaseId') ;;
  *) run=$arg ;;
esac
[ -n "$run" ] || { echo "No successful mac.yml run for $arg"; exit 1; }
gh run view "$run" --json headSha,headBranch,displayTitle -q '"Run '"$run"': \(.headBranch) \(.headSha[:7]) \(.displayTitle)"'
dir=$(mktemp -d)
gh run download "$run" --name BadaIME --dir "$dir"
ditto -x -k "$dir"/BadaIME-*.zip "$dir/app"
make install-prebuilt PREBUILT="$dir/app/BadaIME.app"
rm -rf "$dir"
