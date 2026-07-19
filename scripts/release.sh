#!/usr/bin/env bash
# release.sh — Create a new release
# Usage:
#   ./scripts/release.sh <version>          # Create tag and push
#   ./scripts/release.sh --check            # Verify release readiness
#   ./scripts/release.sh --list             # List recent releases

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

REMOTE="origin"
MAIN_BRANCH="main"

check_release_readiness() {
    echo -e "${CYAN}=== Release Readiness Check ===${NC}"
    
    # Check we're on main
    CURRENT_BRANCH=$(git branch --show-current)
    if [ "$CURRENT_BRANCH" != "$MAIN_BRANCH" ]; then
        echo -e "${RED}✗ Not on $MAIN_BRANCH branch (currently on $CURRENT_BRANCH)${NC}"
        return 1
    fi
    echo -e "${GREEN}✓ On $MAIN_BRANCH branch${NC}"
    
    # Check working tree is clean
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo -e "${RED}✗ Working tree is dirty${NC}"
        return 1
    fi
    echo -e "${GREEN}✓ Working tree is clean${NC}"
    
    # Check we're up to date with remote
    git fetch $REMOTE $MAIN_BRANCH --quiet
    LOCAL_SHA=$(git rev-parse HEAD)
    REMOTE_SHA=$(git rev-parse $REMOTE/$MAIN_BRANCH)
    if [ "$LOCAL_SHA" != "$REMOTE_SHA" ]; then
        echo -e "${RED}✗ Not up to date with remote $MAIN_BRANCH${NC}"
        echo "  Local:  $LOCAL_SHA"
        echo "  Remote: $REMOTE_SHA"
        return 1
    fi
    echo -e "${GREEN}✓ Up to date with remote${NC}"
    
    # Check Cargo.toml version
    VERSION=$(grep '^version' crates/codegen/bucket-bin/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    echo -e "${YELLOW}Current Cargo.toml version: $VERSION${NC}"
    
    # Check for uncommitted changes to version
    if git diff --name-only | grep -q "Cargo.toml"; then
        echo -e "${YELLOW}⚠ Cargo.toml has uncommitted changes${NC}"
    fi
    
    # Check build validation passes
    echo -e "${YELLOW}Running build validation...${NC}"
    if cargo check -p bucket-agent-core --quiet 2>/dev/null; then
        echo -e "${GREEN}✓ agent-core check passes${NC}"
    else
        echo -e "${RED}✗ agent-core check failed${NC}"
        return 1
    fi
    
    if cargo check -p bucket-bin --quiet 2>/dev/null; then
        echo -e "${GREEN}✓ bucket-bin check passes${NC}"
    else
        echo -e "${RED}✗ bucket-bin check failed${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✓ Release readiness check passed${NC}"
    return 0
}

create_release() {
    local VERSION="$1"
    
    # Validate version format
    if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        echo -e "${RED}Invalid version format: $VERSION${NC}"
        echo "Expected: X.Y.Z or X.Y.Z-prerelease (e.g., 0.2.102 or 0.2.102-alpha.1)"
        exit 1
    fi
    
    echo -e "${CYAN}=== Creating release v$VERSION ===${NC}"
    
    # Check readiness
    if ! check_release_readiness; then
        echo -e "${RED}Release readiness check failed${NC}"
        exit 1
    fi
    
    # Create tag
    echo -e "${YELLOW}Creating tag v$VERSION...${NC}"
    git tag -a "v$VERSION" -m "Release v$VERSION"
    
    # Push tag
    echo -e "${YELLOW}Pushing tag...${NC}"
    git push $REMOTE "v$VERSION"
    
    echo -e "${GREEN}✓ Tag v$VERSION created and pushed${NC}"
    echo -e "${CYAN}GitHub Actions will now build and create the release.${NC}"
    echo -e "${CYAN}Monitor at: https://github.com/julesklord/bucket-agent/releases${NC}"
}

list_releases() {
    echo -e "${CYAN}=== Recent Releases ===${NC}"
    git tag -l "v*" --sort=-version:refname | head -10
}

case "${1:-}" in
    --check|-c)
        check_release_readiness
        ;;
    --list|-l)
        list_releases
        ;;
    --help|-h)
        echo "Usage: $0 <version>"
        echo ""
        echo "Commands:"
        echo "  <version>    Create and push a release tag"
        echo "  --check      Verify release readiness"
        echo "  --list       List recent releases"
        ;;
    *)
        if [ -z "${1:-}" ]; then
            echo "Usage: $0 <version>"
            echo "Example: $0 0.2.102"
            exit 1
        fi
        create_release "$1"
        ;;
esac