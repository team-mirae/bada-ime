#!/bin/sh
# Checks the language-key decisions of henkan profiles (ProfileKeyLogic, spec §6.6).
set -e
dir=$(mktemp -d)
cp "$(dirname "$0")/test_profile_keys.swift" "$dir/main.swift"
swiftc "$dir/main.swift" "$(dirname "$0")/../Sources/ProfileKeys.swift" -o "$dir/test"
"$dir/test"
