#!/usr/bin/env python3
"""
TextFSM command output parsing module.

This module provides functions to parse command outputs using TextFSM templates.
It handles template location, selection, and output parsing.
"""

import os
import re
import csv
import logging
import json
from typing import List, Dict, Any, Optional, Union

# Setup logging
logger = logging.getLogger(__name__)

# Path to templates
TEMPLATE_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "textfsm/templates")

# Import TextFSM
from .parse import TextFSM

class NetworkOutputParser:
    """Class for parsing network device command outputs using TextFSM templates."""
    
    def __init__(self, template_dir=None):
        """
        Initialize the parser with a template directory.
    
    Args:
            template_dir: Optional path to template directory. If not provided,
                          uses the default template directory.
        """
        self.template_dir = template_dir or TEMPLATE_DIR
        self._index_loaded = False
        self._platform_templates = {}
        
    def _load_template_index(self):
        """Load the template index file and build a dictionary of templates by platform."""
        if self._index_loaded:
            return self._platform_templates
            
        index_file = os.path.join(self.template_dir, "index")
    if not os.path.isfile(index_file):
        logger.error(f"Index file not found at {index_file}")
            self._index_loaded = True
            return {}
            
        try:
            # Read the index file
            with open(index_file, 'r') as f:
                # Skip comment lines at the beginning
                header_found = False
                header_line = None
                for line in f:
                    line = line.strip()
                    if not line or line.startswith('#'):
                        continue
                    
                    # First non-comment line is the header
                    if not header_found:
                        header_line = line
                        header_found = True
                        break
                
                if not header_found:
                    logger.error("No header found in index file")
                    self._index_loaded = True
                    return {}
                
                # Parse the header to get column positions
                if not header_line:
                    logger.error("No valid header line found in index file")
                    self._index_loaded = True
                    return {}
                
                headers = [h.strip() for h in header_line.split(',')]
                
                # Find the column indices for template, platform, and command
                template_col = -1
                platform_col = -1
                command_col = -1
                
                for i, header in enumerate(headers):
                    if header.lower() == 'template':
                        template_col = i
                    elif header.lower() in ['platform', 'vendor']:
                        platform_col = i
                    elif header.lower() == 'command':
                        command_col = i
                
                if template_col == -1 or platform_col == -1 or command_col == -1:
                    logger.error(f"Missing required columns in index file. Headers: {headers}")
                    self._index_loaded = True
                    return {}
                
                # Reset file pointer to beginning
                f.seek(0)
                
                # Skip header and comments
                header_passed = False
                for line in f:
                    line = line.strip()
                    if not line or line.startswith('#'):
                        continue
                    
                    if not header_passed:
                        header_passed = True
                        continue
                    
                    # Parse the line using CSV parser to handle quoted fields
                    try:
                        row = next(csv.reader([line]))
                        
                        if len(row) <= max(template_col, platform_col, command_col):
                        continue
                        
                        template_file = row[template_col].strip()
                        platform = row[platform_col].strip().lower()
                        command_pattern = row[command_col].strip()
                        
                        # Remove the regex patterns for command completion
                        # Convert sh[[ow]] ver[[sion]] to show version
                        command = re.sub(r'\[\[([^]]+)\]\]', r'\1', command_pattern).lower()
                        
                        # Add to platform templates
                        if platform not in self._platform_templates:
                            self._platform_templates[platform] = []
                        
                        self._platform_templates[platform].append({
                            'command': command,
                            'template': template_file
                        })
                    except Exception as e:
                        logger.debug(f"Error parsing line '{line}': {str(e)}")
                                    continue
                                    
        except Exception as e:
            logger.error(f"Error loading template index: {str(e)}")
            
        self._index_loaded = True
        logger.debug(f"Loaded templates for {len(self._platform_templates)} platforms")
        return self._platform_templates
    
    def find_template(self, platform, command):
        """
        Find the appropriate template for a given platform and command.
        
        Args:
            platform: Device platform (e.g., cisco_ios)
            command: Command string (e.g., show version)
            
        Returns:
            Path to template file or None if not found
        """
        templates = self._load_template_index()
        platform = platform.lower()
        command = command.lower()
        
        if platform not in templates:
            logger.warning(f"No templates found for platform '{platform}'")
            return None
            
        logger.debug(f"Looking for command '{command}' in platform '{platform}' with {len(templates[platform])} templates")
        
        # First try exact match
        for entry in templates[platform]:
            if command == entry['command']:
                template_path = os.path.join(self.template_dir, entry['template'])
                if os.path.isfile(template_path):
                    logger.debug(f"Found exact match template: {template_path}")
                                return template_path
                                
        # Then try substring match
        for entry in templates[platform]:
            if command in entry['command'] or entry['command'] in command:
                template_path = os.path.join(self.template_dir, entry['template'])
                if os.path.isfile(template_path):
                    logger.debug(f"Found substring match template: {template_path}")
                                return template_path
                                
        logger.warning(f"No template found for '{platform}' command '{command}'")
        return None
    
    def parse_output(self, platform, command, data):
    """
    Parse command output using TextFSM.
    
    Args:
            platform: Device platform (e.g., cisco_ios)
            command: Command string (e.g., show version)
            data: Command output as string
        
    Returns:
            List of dictionaries containing parsed data, or None if parsing fails
    """
        if not data:
        logger.warning("Empty output provided for parsing")
        return None
            
        template_path = self.find_template(platform, command)
        if not template_path:
            logger.warning(f"No template found for {platform}, {command}")
            return None
    
    try:
        logger.debug(f"Parsing output using template: {template_path}")
        with open(template_path, 'r') as template_file:
                fsm = TextFSM(template_file)
                parsed_data = fsm.ParseTextToDicts(data)
                print("+++++++++++++++++++++++++++++++++++++++++++++++++++++")
                print(parsed_data)
                print("+++++++++++++++++++++++++++++++++++++++++++++++++++++")
                # Convert to list of dictionaries
                result = []
                for record in parsed_data:
                    record_dict = {}
                    for i, header in enumerate(fsm.header):
                        record_dict[header] = record[i]
                    result.append(record_dict)
                
                return result
    except Exception as e:
        logger.error(f"Error parsing output: {str(e)}")
        return None

    def parse_to_json(self, platform, command, data):
        """
        Parse command output and return as JSON string.
        
        Args:
            platform: Device platform (e.g., cisco_ios)
            command: Command string (e.g., show version)
            data: Command output as string
            
        Returns:
            JSON string of parsed data, or None if parsing fails
        """
        result = self.parse_output(platform, command, data)
        if result:
            return json.dumps(result, indent=2)
        return None

# For backward compatibility
parser = NetworkOutputParser()

def parse_output(platform, command, data):
    """
    Parse command output using TextFSM (function interface).
    
    Args:
        platform: Device platform (e.g., cisco_ios)
        command: Command string (e.g., show version)
        data: Command output as string
        
    Returns:
        List of dictionaries containing parsed data, or None if parsing fails
    """
    return parser.parse_output(platform, command, data)

def parse_output_to_json(platform, command, data):
    """
    Parse command output and return as JSON string (function interface).
    
    Args:
        platform: Device platform (e.g., cisco_ios)
        command: Command string (e.g., show version)
        data: Command output as string
        
    Returns:
        JSON string of parsed data, or None if parsing fails
    """
    return parser.parse_to_json(platform, command, data)

# Make these functions available when imported
__all__ = ['parse_output', 'parse_output_to_json', 'NetworkOutputParser'] 