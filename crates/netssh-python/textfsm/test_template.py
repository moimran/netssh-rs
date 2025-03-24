#!/usr/bin/env python3
"""Simple script to test if a command finds a TextFSM template."""

import sys
import os
import logging
import re

# Add parent directory to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from textfsm.parse_output import _find_template, _load_platform_templates, TEMPLATE_DIR, normalize_command, clear_template_cache, _load_template_index

# Configure logging
logging.basicConfig(level=logging.DEBUG, 
                   format='DEBUG: %(message)s')
logger = logging.getLogger()

def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <platform> <command> [--force-reload] [--debug-index]")
        print(f"       --force-reload: Force reload templates from index file (clear cache)")
        print(f"       --debug-index: Print detailed debug info about the index file")
        sys.exit(1)
    
    platform = sys.argv[1].lower()
    command = sys.argv[2]
    force_reload = "--force-reload" in sys.argv
    debug_index = "--debug-index" in sys.argv
    
    if force_reload:
        print("Forcing reload of template cache")
        clear_template_cache()
    
    print(f"Looking for template with:")
    print(f"  Platform: '{platform}'")
    print(f"  Command: '{command}'")
    print(f"  Template directory: {TEMPLATE_DIR}")
    
    # Check if index file exists
    index_file = os.path.join(TEMPLATE_DIR, "index")
    if os.path.exists(index_file):
        print(f"  Index file: Found at {index_file}")
    else:
        print(f"  Index file: NOT FOUND at {index_file}")
        return 1
        
    # Debug the index file - show specific platform entries
    if debug_index:
        with open(index_file, 'r') as f:
            content = f.read()
            
        print("\n=== INDEX FILE ANALYSIS ===")
        
        # Count total lines and non-comment lines
        lines = content.split('\n')
        non_comment_lines = [line for line in lines if line.strip() and not line.strip().startswith('#')]
        print(f"Total lines in index file: {len(lines)}")
        print(f"Non-comment lines: {len(non_comment_lines)}")
        
        # Find all platform entries
        platform_pattern = re.compile(f'[^,]+, [^,]+, {platform}(,| )')
        platform_lines = [line for line in lines if platform_pattern.search(line)]
        print(f"\nFound {len(platform_lines)} entries for platform '{platform}':")
        
        # Show a sample of them
        for i, line in enumerate(platform_lines[:10]):
            print(f"  {i+1}. {line.strip()}")
        if len(platform_lines) > 10:
            print(f"  ... and {len(platform_lines) - 10} more")
            
        # Find version command specifically
        version_pattern = re.compile(f'[^,]+, [^,]+, {platform},(.*ver|.*version|.*VER)')
        version_lines = [line for line in lines if version_pattern.search(line.lower())]
        print(f"\nFound {len(version_lines)} version-related entries for platform '{platform}':")
        for line in version_lines:
            print(f"  {line.strip()}")
    
    # Show normalized command
    norm_command = normalize_command(command)
    if norm_command != command:
        print(f"  Normalized command: '{norm_command}'")
    
    # Load the platform index and get patterns for this platform
    template_index = _load_template_index()
    platform_patterns = template_index.get(platform, [])
    
    print(f"\nFound {len(platform_patterns)} pattern(s) for platform '{platform}':")
    for i, (regex, template_file) in enumerate(platform_patterns[:5], 1):
        print(f"  {i}. Pattern: '{regex.pattern}' → Template: {template_file}")
    if len(platform_patterns) > 5:
        print(f"  ... and {len(platform_patterns) - 5} more")
    
    # Test each pattern against our command
    if platform_patterns:
        print("\nPattern matching tests:")
        for i, (regex, template_file) in enumerate(platform_patterns):
            # Test if regex matches the command
            match = regex.match(norm_command)
            
            # Report match results
            print(f"  Pattern {i+1}: {template_file}")
            print(f"    Regex: '{regex.pattern}'")
            print(f"    Match against '{norm_command}': {'YES' if match else 'NO'}")
            if match:
                print(f"    Match groups: {match.groups() if match.groups() else 'None'}")
    
    # Search for template
    template_path = _find_template(platform, command)
    
    if template_path:
        print(f"\nSUCCESS: Template found: {template_path}")
        print(f"Template filename: {os.path.basename(template_path)}")
        return 0
    else:
        print(f"\nFAILURE: No template found for platform '{platform}' and command '{command}'")
        
        if len(platform_patterns) == 0:
            print("\nNo pattern definitions available for this platform.")
        
        return 1

if __name__ == "__main__":
    sys.exit(main()) 