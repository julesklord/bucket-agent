#!/usr/bin/env bash
set -euo pipefail

# Phase 1 - Nominal Cleanup Script
# Renames all bucket-* crates to bucket-*
# Updates all references in Cargo.toml and Rust source files

ROOT_DIR="/mnt/DEV/Proyectos/repos/julesklord/bucket-agent"
cd "$ROOT_DIR"

# Create mapping from the JSON file
declare -A CRATE_MAP=(
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
)

# Directories that should NOT be renamed (third_party, prod, etc.)
SKIP_DIRS=("third_party" "prod" "target" ".git" ".github" "node_modules")

# Step 1: Rename crate directories
echo "=== Step 1: Renaming crate directories ==="
for old_name in "${!CRATE_MAP[@]}"; do
    new_name="${CRATE_MAP[$old_name]}"
    
    # Find the directory
    old_dir=$(find . -maxdepth 4 -type d -name "$old_name" 2>/dev/null | grep -v target | grep -v ".git" | head -1)
    
    if [[ -n "$old_dir" && -d "$old_dir" ]]; then
        new_dir=$(dirname "$old_dir")/"$new_name"
        if [[ "$old_dir" != "$new_dir" ]]; then
            echo "Renaming directory: $old_dir -> $new_dir"
            mv "$old_dir" "$new_dir"
        fi
    else
        echo "WARNING: Directory for $old_name not found"
    fi
done

echo "=== Step 2: Updating package names in Cargo.toml files ==="
# Update each crate's Cargo.toml name field
for old_name in "${!CRATE_MAP[@]}"; do
    new_name="${CRATE_MAP[$old_name]}"
    
    # Find the Cargo.toml
    cargo_toml=$(find . -maxdepth 4 -path "*/$new_name/Cargo.toml" 2>/dev/null | grep -v target | grep -v ".git" | head -1)
    
    if [[ -n "$cargo_toml" && -f "$cargo_toml" ]]; then
        echo "Updating name in: $cargo_toml"
        # Update the name field in [package]
        sed -i "s/^name = \"$old_name\"/name = \"$new_name\"/" "$cargo_toml"
    else
        echo "WARNING: Cargo.toml for $new_name not found"
    fi
done

echo "=== Step 3: Updating internal dependencies in all Cargo.toml ==="
# Find all Cargo.toml files and update dependency references
find . -name "Cargo.toml" -type f | grep -v target | grep -v ".git" | while read cargo_toml; do
    echo "Processing dependencies in: $cargo_toml"
    
    for old_name in "${!CRATE_MAP[@]}"; do
        new_name="${CRATE_MAP[$old_name]}"
        
        # Convert to Cargo.toml dependency format (hyphens to underscores for some cases)
        old_dep=$(echo "$old_name" | sed 's/-/_/g')
        new_dep=$(echo "$new_name" | sed 's/-/_/g')
        
        # Update path dependencies in [dependencies], [dev-dependencies], [build-dependencies]
        # Format: bucket-agent-core = { path = "..." }
        sed -i "s/^${old_name} = { path = /${new_name} = { path = /g" "$cargo_toml"
        sed -i "s/^${old_dep} = { path = /${new_dep} = { path = /g" "$cargo_toml"
        
        # Also handle version dependencies (if any internal crate uses version)
        sed -i "s/^${old_name} = \"[^\"]*\"/${new_name} = { path = \"..\" }/g" "$cargo_toml"
    done
done

echo "=== Step 4: Updating root Cargo.toml ==="
ROOT_CARGO="Cargo.toml"

# Update members list
for old_name in "${!CRATE_MAP[@]}"; do
    new_name="${CRATE_MAP[$old_name]}"
    
    # Find the member path
    old_member=$(grep -E "^\s+\"[^\"]*${old_name}\"" "$ROOT_CARGO" | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
    if [[ -n "$old_member" ]]; then
        new_member=$(echo "$old_member" | sed "s/${old_name}/${new_name}/")
        echo "Updating member: $old_member -> $new_member"
        sed -i "s|\"$old_member\"|\"$new_member\"|g" "$ROOT_CARGO"
    fi
done

# Update workspace.dependencies paths
for old_name in "${!CRATE_MAP[@]}"; do
    new_name="${CRATE_MAP[$old_name]}"
    
    old_dep_line=$(grep -E "^\s+${old_name} = { path = " "$ROOT_CARGO" | head -1)
    if [[ -n "$old_dep_line" ]]; then
        new_dep_line=$(echo "$old_dep_line" | sed "s/${old_name}/${new_name}/g" | sed "s|${old_name}|${new_name}|g")
        # Also need to update the path
        new_dep_line=$(echo "$new_dep_line" | sed "s|${old_name}|${new_name}|g")
        echo "Updating workspace dep: $old_name -> $new_name"
        sed -i "s|${old_name} = { path = \"[^\"]*${old_name}[^\"]*\"|${new_name} = { path = \"$(echo "$old_dep_line" | sed -n 's/.*path = \"\([^"]*\)\".*/\1/p' | sed "s/${old_name}/${new_name}/")\"|g" "$ROOT_CARGO"
    fi
done

echo "=== Step 5: Updating Rust source code ==="
# Update use statements and qualified paths in .rs files
find . -name "*.rs" -type f | grep -v target | grep -v ".git" | while read rs_file; do
    modified=false
    
    for old_name in "${!CRATE_MAP[@]}"; do
        new_name="${CRATE_MAP[$old_name]}"
        
        old_underscore=$(echo "$old_name" | sed 's/-/_/g')
        new_underscore=$(echo "$new_name" | sed 's/-/_/g')
        
        # Update use statements: use bucket_agent_core:: -> use bucket_agent_core::
        if grep -q "use ${old_underscore}::" "$rs_file" 2>/dev/null; then
            sed -i "s/use ${old_underscore}::/use ${new_underscore}::/g" "$rs_file"
            modified=true
        fi
        
        # Update extern crate statements
        if grep -q "extern crate ${old_underscore};" "$rs_file" 2>/dev/null; then
            sed -i "s/extern crate ${old_underscore};/extern crate ${new_underscore};/g" "$rs_file"
            modified=true
        fi
        
        # Update qualified paths: bucket_agent_core::some::path
        if grep -q "${old_underscore}::" "$rs_file" 2>/dev/null; then
            sed -i "s/${old_underscore}::/${new_underscore}::/g" "$rs_file"
            modified=true
        fi
        
        # Update macro imports: use bucket_agent_core_macros::
        if grep -q "use ${old_underscore}_macros::" "$rs_file" 2>/dev/null; then
            sed -i "s/use ${old_underscore}_macros::/use ${new_underscore}_macros::/g" "$rs_file"
            modified=true
        fi
    done
    
    if [[ "$modified" == "true" ]]; then
        echo "Updated: $rs_file"
    fi
done

echo "=== Step 6: Updating environment variable references ==="
# Update GROK_* -> BUCKET_* in source files
find . -name "*.rs" -type f | grep -v target | grep -v ".git" | while read rs_file; do
    # Skip if file doesn't contain GROK_
    if grep -q "GROK_" "$rs_file" 2>/dev/null; then
        # We'll do specific replacements
        sed -i 's/BUCKET_HOME/BUCKET_HOME/g' "$rs_file"
        sed -i 's/BUCKET_LOG_FILE/BUCKET_LOG_FILE/g' "$rs_file"
        sed -i 's/BUCKET_AUTH_PROVIDER_COMMAND/BUCKET_AUTH_PROVIDER_COMMAND/g' "$rs_file"
        sed -i 's/BUCKET_OIDC_ISSUER/BUCKET_OIDC_ISSUER/g' "$rs_file"
        sed -i 's/BUCKET_MODELS_BASE_URL/BUCKET_MODELS_BASE_URL/g' "$rs_file"
        sed -i 's/BUCKET_API_KEY/BUCKET_API_KEY/g' "$rs_file"
        echo "Updated env vars in: $rs_file"
    fi
done

# Also update in documentation and config files
find . \( -name "*.md" -o -name "*.toml" -o -name "*.json" -o -name "*.sh" \) -type f | grep -v target | grep -v ".git" | while read file; do
    if grep -q "GROK_" "$file" 2>/dev/null; then
        sed -i 's/BUCKET_HOME/BUCKET_HOME/g' "$file"
        sed -i 's/BUCKET_LOG_FILE/BUCKET_LOG_FILE/g' "$file"
        sed -i 's/BUCKET_AUTH_PROVIDER_COMMAND/BUCKET_AUTH_PROVIDER_COMMAND/g' "$file"
        sed -i 's/BUCKET_OIDC_ISSUER/BUCKET_OIDC_ISSUER/g' "$file"
        sed -i 's/BUCKET_MODELS_BASE_URL/BUCKET_MODELS_BASE_URL/g' "$file"
        echo "Updated env vars in: $file"
    fi
    if grep -q "BUCKET_API_KEY" "$file" 2>/dev/null; then
        sed -i 's/BUCKET_API_KEY/BUCKET_API_KEY/g' "$file"
        echo "Updated BUCKET_API_KEY in: $file"
    fi
done

echo "=== Step 7: Updating config paths (~/.bucket -> ~/.bucket) ==="
find . -type f \( -name "*.rs" -o -name "*.md" -o -name "*.toml" -o -name "*.json" -o -name "*.sh" \) | grep -v target | grep -v ".git" | while read file; do
    if grep -q '\.grok' "$file" 2>/dev/null; then
        sed -i 's|~/\.grok|~/.bucket|g' "$file"
        sed -i 's|\.bucket/|.bucket/|g' "$file"
        sed -i 's|/\.grok|/.bucket|g' "$file"
        echo "Updated config paths in: $file"
    fi
done

echo "=== Done! ==="
echo "Run 'cargo check' to verify the changes."