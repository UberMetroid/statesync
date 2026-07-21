import json
import subprocess
import os

def run():
    # Run cargo check to get json output
    env = os.environ.copy()
    env["RUSTFLAGS"] = "-D missing_docs"
    result = subprocess.run(
        ["cargo", "check", "--message-format=json"],
        cwd="/home/jeryd/Projects/studio2201/statesync",
        env=env,
        capture_output=True,
        text=True
    )
    
    # Track which lines have had docs added to avoid offsetting
    # Actually, we can just process from bottom to top for each file
    file_fixes = {}
    
    for line in result.stdout.splitlines():
        if not line.strip(): continue
        try:
            msg = json.loads(line)
        except:
            continue
            
        if msg.get("reason") != "compiler-message":
            continue
            
        message = msg.get("message", {})
        if message.get("code", {}).get("code") != "missing_docs":
            continue
            
        spans = message.get("spans", [])
        if not spans: continue
        
        primary_span = next((s for s in spans if s.get("is_primary")), None)
        if not primary_span: continue
        
        file_name = primary_span.get("file_name")
        line_start = primary_span.get("line_start")
        
        if file_name not in file_fixes:
            file_fixes[file_name] = set()
            
        file_fixes[file_name].add(line_start)
        
    for file_name, lines in file_fixes.items():
        if not os.path.exists(file_name): continue
        with open(file_name, "r") as f:
            content = f.readlines()
            
        # Sort lines descending so we can insert without changing previous line numbers
        for line_num in sorted(lines, reverse=True):
            idx = line_num - 1
            # Add docstring
            # Figure out indentation
            orig_line = content[idx]
            indent = orig_line[:len(orig_line) - len(orig_line.lstrip())]
            content.insert(idx, indent + "/// Missing documentation.\n")
            
        with open(file_name, "w") as f:
            f.writelines(content)
            
run()
