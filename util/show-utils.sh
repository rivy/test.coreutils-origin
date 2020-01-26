#!/bin/sh

# spell-checker:ignore uutils
# spell-checker:ignore (utils) cksum dircolors hashsum mkdir mktemp printf printenv readlink realpath relpath rmdir shuf tsort unexpand
# spell-checker:ignore (jq) deps startswith

# default (fully-cross platform) utility list
default_utils="base32 base64 basename cat cksum comm cp cut date dircolors dirname echo env expand expr factor false fmt fold hashsum head join link ln ls mkdir mktemp more mv nl od paste printenv printf ptx pwd readlink realpath relpath rm rmdir seq shred shuf sleep sort split sum tac tail tee test tr true truncate tsort unexpand uniq wc yes"

# `jq` available?
unset JQ
jq --version 1>/dev/null 2>&1
if [ $? -eq 0 ]; then export JQ="jq"; fi

if [ -z "${JQ}" ]; then
    echo 'WARN: missing `jq` (install with `sudo apt install jq`); falling back to default (only fully cross-platform) utility list' 1>&2
    echo $default_utils
else
    cargo metadata $* --format-version 1 | jq -r "[.resolve.nodes[] | {id: .id, deps: [.deps[].name]}] | .[] | select(.id|startswith(\"uutils\")) | [.deps[] | select(startswith(\"uu_\"))] | [.[] | sub(\"^uu_\"; \"\")] | join(\" \")"
fi
