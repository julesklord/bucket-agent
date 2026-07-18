#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="/mnt/DEV/Proyectos/repos/julesklord/bucket-agent"
cd "$ROOT_DIR"

echo "=== Fixing Cargo.toml dependency references ==="

# Map of old dependency names to new ones
declare -A DEP_MAP=(
    # bucket-* -> bucket-*
    ["bucket-agent-core"]="bucket-agent-core"
    ["bucket-tui"]="bucket-tui"
    ["bucket-bin"]="bucket-bin"
    ["bucket-tools"]="bucket-tools"
    ["bucket-workspace"]="bucket-workspace"
    ["bucket-telemetry"]="bucket-telemetry"
    ["bucket-auth"]="bucket-auth"
    ["bucket-memory"]="bucket-memory"
    ["bucket-markdown"]="bucket-markdown"
    ["bucket-config"]="bucket-config"
    ["bucket-updater"]="bucket-updater"
    ["bucket-agent-base"]="bucket-agent-base"
    ["bucket-acp"]="bucket-acp"
    ["bucket-hub-core"]="bucket-hub-core"
    ["bucket-hub-mcp-adapter"]="bucket-hub-mcp-adapter"
    ["bucket-hub-sdk"]="bucket-hub-sdk"
    ["bucket-compaction"]="bucket-compaction"
    ["bucket-announcements"]="bucket-announcements"
    ["bucket-agent"]="bucket-agent"
    ["bucket-config-types"]="bucket-config-types"
    ["bucket-env"]="bucket-env"
    ["bucket-hooks"]="bucket-hooks"
    ["bucket-http"]="bucket-http"
    ["bucket-markdown-core"]="bucket-markdown-core"
    ["bucket-mcp"]="bucket-mcp"
    ["bucket-mermaid"]="bucket-mermaid"
    ["bucket-models"]="bucket-models"
    ["bucket-sampler"]="bucket-sampler"
    ["bucket-sampling-types"]="bucket-sampling-types"
    ["bucket-sandbox"]="bucket-sandbox"
    ["bucket-secrets"]="bucket-secrets"
    ["bucket-shared"]="bucket-shared"
    ["bucket-shell-session-support"]="bucket-shell-session-support"
    ["bucket-subagent-resolution"]="bucket-subagent-resolution"
    ["bucket-test-support"]="bucket-test-support"
    ["bucket-tools-api"]="bucket-tools-api"
    ["bucket-version"]="bucket-version"
    ["bucket-voice"]="bucket-voice"
    ["bucket-workspace-client"]="bucket-workspace-client"
    ["bucket-workspace-types"]="bucket-workspace-types"
    ["bucket-tui-minimal"]="bucket-tui-minimal"
    ["bucket-tui-pty-harness"]="bucket-tui-pty-harness"
    ["bucket-tui-render"]="bucket-tui-render"
    ["bucket-paths"]="bucket-paths"
    ["bucket-plugin-marketplace"]="bucket-plugin-marketplace"
    ["bucket-hooks-plugins-types"]="bucket-hooks-plugins-types"
    ["bucket-fast-worktree"]="bucket-fast-worktree"
    ["bucket-file-utils"]="bucket-file-utils"
    ["bucket-fsnotify"]="bucket-fsnotify"
    ["bucket-gix-status"]="bucket-gix-status"
    ["bucket-hunk-tracker"]="bucket-hunk-tracker"
    ["bucket-mixpanel"]="bucket-mixpanel"
    ["bucket-prompt-queue"]="bucket-prompt-queue"
    ["bucket-ratatui-inline"]="bucket-ratatui-inline"
    ["bucket-ratatui-textarea"]="bucket-ratatui-textarea"
    ["bucket-sqlite-journal"]="bucket-sqlite-journal"
    ["bucket-system-power"]="bucket-system-power"
    ["bucket-token-estimation"]="bucket-token-estimation"
    ["bucket-tracing-macros"]="bucket-tracing-macros"
    ["bucket-tty-utils"]="bucket-tty-utils"
    ["bucket-interjection-core"]="bucket-interjection-core"
    ["bucket-circuit-breaker"]="bucket-circuit-breaker"
    ["bucket-test-utils"]="bucket-test-utils"
    ["bucket-tool-protocol"]="bucket-tool-protocol"
    ["bucket-tool-runtime"]="bucket-tool-runtime"
    ["bucket-tool-types"]="bucket-tool-types"
    ["bucket-tracing"]="bucket-tracing"
    ["bucket-proto-build"]="bucket-proto-build"
    ["bucket-chat-state"]="bucket-chat-state"
    ["bucket-codebase-graph"]="bucket-codebase-graph"
    ["bucket-crash-handler"]="bucket-crash-handler"
    ["bucket-agent-lifecycle"]="bucket-agent-lifecycle"
    ["bucket-mcp"]="bucket-mcp"
    ["bucket-agent-core"]="bucket-agent-core"
    ["bucket-tui"]="bucket-tui"
    # bucket-* (non-grok) -> bucket-*
    ["bucket-"]="bucket-"
)

# Find all Cargo.toml files in the workspace (excluding third_party, target, .git)
find . -name "Cargo.toml" -type f | grep -v target | grep -v ".git" | grep -v "third_party" | while read cargo_file; do
    modified=false
    
    for old_dep in "${!DEP_MAP[@]}"; do
        new_dep="${DEP_MAP[$old_dep]}"
        
        # Replace workspace = true dependencies
        if grep -q "${old_dep} = { workspace = true }" "$cargo_file" 2>/dev/null; then
            sed -i "s/${old_dep} = { workspace = true }/${new_dep} = { workspace = true }/g" "$cargo_file"
            modified=true
        fi
        
        # Replace path dependencies with old names
        if grep -q "path = \".*/${old_dep}\"" "$cargo_file" 2>/dev/null; then
            sed -i "s|path = \"\.\./${old_dep}\"|path = \"../${new_dep}\"|g" "$cargo_file"
            sed -i "s|path = \"\.\./\.\./${old_dep}\"|path = \"../../${new_dep}\"|g" "$cargo_file"
            modified=true
        fi
        
        # Replace version dependencies with path
        if grep -q "^${old_dep} = " "$cargo_file" 2>/dev/null; then
            sed -i "s|^${old_dep} = .*|${new_dep} = { workspace = true }|g" "$cargo_file"
            modified=true
        fi
        
        # Replace features references like dep:bucket-*
        if grep -q "dep:${old_dep}" "$cargo_file" 2>/dev/null; then
            sed -i "s/dep:${old_dep}/dep:${new_dep}/g" "$cargo_file"
            modified=true
        fi
    done
    
    if [[ "$modified" == "true" ]]; then
        echo "Updated: $cargo_file"
    fi
done

echo "=== Fixing path dependencies that still have bucket-* in path ==="
# Additional pass for path dependencies
find . -name "Cargo.toml" -type f | grep -v target | grep -v ".git" | grep -v "third_party" | while read cargo_file; do
    if grep -q 'path = ".*bucket-' "$cargo_file" 2>/dev/null; then
        sed -i 's|path = "\.\./bucket-|path = "../bucket-|g' "$cargo_file"
        sed -i 's|path = "\.\./\.\./bucket-|path = "../../bucket-|g' "$cargo_file"
        echo "Fixed paths in: $cargo_file"
    fi
    if grep -q 'path = ".*bucket-' "$cargo_file" 2>/dev/null; then
        sed -i 's|path = "\.\./bucket-|path = "../bucket-|g' "$cargo_file"
        sed -i 's|path = "\.\./\.\./bucket-|path = "../../bucket-|g' "$cargo_file"
        echo "Fixed bucket- paths in: $cargo_file"
    fi
done

echo "=== Fixing bucket-ratatui-textarea references ==="
# Special case for bucket-ratatui-textarea
find . -name "Cargo.toml" -type f | grep -v target | grep -v ".git" | grep -v "third_party" | while read cargo_file; do
    if grep -q "bucket-ratatui-textarea" "$cargo_file" 2>/dev/null; then
        sed -i 's/bucket-ratatui-textarea/bucket-ratatui-textarea/g' "$cargo_file"
        echo "Fixed ratatui-textarea in: $cargo_file"
    fi
done

echo "=== Fixing bucket-agent-core references in features ==="
find . -name "Cargo.toml" -type f | grep -v target | grep -v ".git" | grep -v "third_party" | while read cargo_file; do
    if grep -q "bucket-agent-core" "$cargo_file" 2>/dev/null; then
        sed -i 's/bucket-agent-core/bucket-agent-core/g' "$cargo_file"
        echo "Fixed bucket-agent-core in: $cargo_file"
    fi
done

echo "=== Fixing bucket-tui references in features ==="
find . -name "Cargo.toml" -type f | grep -v target | grep -v ".git" | grep -v "third_party" | while read cargo_file; do
    if grep -q "bucket-tui" "$cargo_file" 2>/dev/null; then
        sed -i 's/bucket-tui/bucket-tui/g' "$cargo_file"
        echo "Fixed bucket-tui in: $cargo_file"
    fi
done

echo "=== Fixing bucket-env/test-support feature ==="
find . -name "Cargo.toml" -type f | grep -v target | grep -v ".git" | grep -v "third_party" | while read cargo_file; do
    if grep -q "bucket-env/test-support" "$cargo_file" 2>/dev/null; then
        sed -i 's/bucket-env\/test-support/bucket-env\/test-support/g' "$cargo_file"
        echo "Fixed test-support feature in: $cargo_file"
    fi
done

echo "=== Done fixing Cargo.toml ==="