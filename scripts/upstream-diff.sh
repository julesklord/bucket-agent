#!/usr/bin/env bash
# upstream-diff.sh — Extract and analyze diffs from upstream
# Usage:
#   ./scripts/upstream-diff.sh                    # Show summary
#   ./scripts/upstream-diff.sh list               # List changed files
#   ./scripts/upstream-diff.sh category <cat>     # Show files in a category
#   ./scripts/upstream-diff.sh diff <file>        # Show diff for a specific file
#   ./scripts/upstream-diff.sh filter <pattern>   # Filter files by pattern (grep)

set -euo pipefail

UPSTREAM="${UPSTREAM_REF:-upstream/main}"
LOCAL="${LOCAL_REF:-main}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Get diff stats
get_diff_files() {
    git diff --name-status "$LOCAL".."$UPSTREAM" 2>/dev/null || \
    git diff --name-status "main".."$UPSTREAM"
}

case "${1:-summary}" in
    summary)
        echo -e "${CYAN}=== Upstream Diff Summary ===${NC}"
        echo ""
        
        get_diff_files | awk '{
            status = $1
            if (status == "A") added++
            else if (status == "D") deleted++
            else if (status == "R") renamed++
            else modified++
            
            # Extract crate/area from path
            path = $2
            if (match(path, /crates\/codegen\/([^/]+)/, arr)) {
                crates[arr[1]]++
            }
        }
        END {
            printf "  Added:    %d\n", added+0
            printf "  Deleted:  %d\n", deleted+0
            printf "  Renamed:  %d\n", renamed+0
            printf "  Modified: %d\n", modified+0
            printf "  Total:    %d files\n\n", added+deleted+renamed+modified
            
            print "By crate:"
            for (c in crates) {
                printf "  %-35s %d\n", c, crates[c]
            }
        }'
        ;;
        
    list)
        echo -e "${CYAN}=== Changed Files ===${NC}"
        get_diff_files | sort -k2
        ;;
        
    category)
        CATEGORY="${2:-}"
        if [ -z "$CATEGORY" ]; then
            echo "Categories: codegen, common, build, tests, config, proto, other"
            exit 1
        fi
        
        case "$CATEGORY" in
            codegen) get_diff_files | grep -E "crates/codegen/" ;;
            common)  get_diff_files | grep -E "crates/common/" ;;
            build)   get_diff_files | grep -E "crates/build/" ;;
            tests)   get_diff_files | grep -E "_test\.rs|tests/" ;;
            config)  get_diff_files | grep -E "Cargo\.toml|config|\.json$" ;;
            proto)   get_diff_files | grep -E "\.proto$" ;;
            other)   get_diff_files | grep -vE "crates/(codegen|common|build)/" ;;
        esac
        ;;
        
    diff)
        FILE="${2:-}"
        if [ -z "$FILE" ]; then
            echo "Usage: $0 diff <file-path>"
            exit 1
        fi
        git diff "$LOCAL".."$UPSTREAM" -- "$FILE"
        ;;
        
    filter)
        PATTERN="${2:-}"
        if [ -z "$PATTERN" ]; then
            echo "Usage: $0 filter <grep-pattern>"
            exit 1
        fi
        get_diff_files | grep -i "$PATTERN"
        ;;
        
    categories)
        echo -e "${CYAN}=== File Categories ===${NC}"
        get_diff_files | awk '{
            path = $2
            if (match(path, /crates\/codegen\/([^/]+)/, arr)) {
                print arr[1]
            }
        }' | sort | uniq -c | sort -rn
        ;;
        
    *)
        echo "Usage: $0 {summary|list|category|diff|filter|categories}"
        echo ""
        echo "Commands:"
        echo "  summary              Show diff summary with stats"
        echo "  list                 List all changed files"
        echo "  category <cat>       Show files in category (codegen|common|build|tests|config|proto|other)"
        echo "  diff <file>          Show diff for a specific file"
        echo "  filter <pattern>     Filter files by pattern"
        echo "  categories           Show all crate categories with counts"
        exit 1
        ;;
esac