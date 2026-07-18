#!/usr/bin/env bash
# extract-diff.sh — Extract diff for specific crates/files for surgical review
# Usage:
#   ./scripts/extract-diff.sh <crate-name>           # Extract full diff for a crate
#   ./scripts/extract-diff.sh <crate-name> --stat     # Show stat only
#   ./scripts/extract-diff.sh <file-path>            # Extract diff for a specific file

set -euo pipefail

UPSTREAM="${UPSTREAM_REF:-upstream/main}"
LOCAL="${LOCAL_REF:-main}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

if [ $# -lt 1 ]; then
    echo "Usage: $0 <crate-or-file> [--stat]"
    exit 1
fi

TARGET="$1"
STAT_ONLY="${2:-}"

# Check if it's a file or crate
if [[ "$TARGET" == *"/"* ]]; then
    # It's a file path
    echo -e "${CYAN}=== Diff for file: $TARGET ===${NC}"
    git diff "$LOCAL".."$UPSTREAM" -- "$TARGET"
else
    # It's a crate name
    echo -e "${CYAN}=== Diff for crate: $TARGET ===${NC}"
    echo ""
    
    if [ "$STAT_ONLY" = "--stat" ]; then
        git diff --stat "$LOCAL".."$UPSTREAM" -- "crates/codegen/$TARGET/" "crates/common/$TARGET/" "crates/build/$TARGET/"
    else
        # Show files changed
        echo -e "${YELLOW}Files changed:${NC}"
        git diff --name-status "$LOCAL".."$UPSTREAM" -- "crates/codegen/$TARGET/" "crates/common/$TARGET/" "crates/build/$TARGET/"
        echo ""
        
        # Show full diff
        echo -e "${YELLOW}Full diff:${NC}"
        git diff "$LOCAL".."$UPSTREAM" -- "crates/codegen/$TARGET/" "crates/common/$TARGET/" "crates/build/$TARGET/"
    fi
fi