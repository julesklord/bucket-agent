#!/usr/bin/env python3
import os
import re

def main():
    # Root directory is the parent of the scripts folder
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    cargo_toml_path = os.path.join(root_dir, "Cargo.toml")
    
    # Standard subdirectories where workspace crates are located
    search_dirs = ["crates/build", "crates/codegen", "crates/common", "prod/mc", "third_party"]
    
    members = []
    for sdir in search_dirs:
        full_sdir = os.path.join(root_dir, sdir)
        if not os.path.isdir(full_sdir):
            continue
        # Scan for subdirectories with a Cargo.toml
        for item in sorted(os.listdir(full_sdir)):
            item_path = os.path.join(full_sdir, item)
            if os.path.isdir(item_path):
                if os.path.isfile(os.path.join(item_path, "Cargo.toml")):
                    # Get the relative path from the root
                    rel_path = os.path.relpath(item_path, root_dir)
                    members.append(rel_path)

    # Sort members for consistency and deterministic output
    members.sort()
    
    # Read the current Cargo.toml
    if not os.path.isfile(cargo_toml_path):
        print(f"Error: {cargo_toml_path} not found.")
        return

    with open(cargo_toml_path, "r", encoding="utf-8") as f:
        content = f.read()
        
    # Regex to match the members array in the [workspace] section
    pattern = r"(members\s*=\s*\[)(.*?)(\s*\])"
    
    def replacer(match):
        prefix = match.group(1)
        suffix = match.group(3)
        formatted_members = "\n" + "".join(f'    "{m}",\n' for m in members)
        return f"{prefix}{formatted_members}{suffix}"
        
    new_content, count = re.subn(pattern, replacer, content, flags=re.DOTALL)
    if count == 0:
        print("Error: Could not locate the 'members' array inside Cargo.toml")
        return
        
    with open(cargo_toml_path, "w", encoding="utf-8") as f:
        f.write(new_content)
        
    print(f"Successfully updated Cargo.toml with {len(members)} workspace members.")

if __name__ == "__main__":
    main()
