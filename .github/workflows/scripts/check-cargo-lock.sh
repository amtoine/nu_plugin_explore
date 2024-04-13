#!/usr/bin/env bash

[[ -n "$(git status --short Cargo.lock)" ]] && exit 1

exit 0
