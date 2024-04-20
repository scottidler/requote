#!/usr/bin/env bash

if [ -n "$DEBUG" ]; then
	set -x
fi

if [ $# -eq 0 ]; then
	echo "Usage: $0 <file>"
	exit 1
fi

FILE="$1"
shift 1
bat -p "$FILE"
target/debug/requote "$@" "$FILE"
bat -p "$FILE.new"
