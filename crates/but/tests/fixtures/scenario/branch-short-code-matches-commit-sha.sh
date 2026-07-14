#!/usr/bin/env bash

set -eu -o pipefail

source "${BASH_SOURCE[0]%/*}/shared.sh"

### General Description

# A branch has the short code `br`, and its commit has no change ID and a SHA
# beginning with `b0`.
git-init-frozen
commit-file M
setup_target_to_match_main

git checkout -b branch
  echo branch >branch
  git add branch

  tree=$(git write-tree)
  parent=$(git rev-parse main)
  commit_id=$(git commit-tree "$tree" -p "$parent" -m "add branch 814")
  if [[ "$commit_id" != b0* ]]; then
    echo "BUG: expected commit ID to start with b0, got $commit_id" >&2
    exit 1
  fi
  git update-ref refs/heads/branch "$commit_id"

create_workspace_commit_once branch
