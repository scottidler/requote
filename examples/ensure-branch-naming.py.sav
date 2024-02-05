#!/bin/env python3

import subprocess
import sys
import re

def get_current_branch():
    # Run git command to get the current branch name
    result = subprocess.run(['git', 'rev-parse', '--abbrev-ref', 'HEAD'], capture_output=True, text=True)
    if result.returncode != 0:
        print("Error: Unable to get current branch name.")
        sys.exit(1)
    return result.stdout.strip()

def is_branch_name_valid(branch_name):
    # Regex to match the branch naming convention
    # At least 2 uppercase alphabetic characters or numbers, followed by a dash and a number,
    # and then a slash and some text.
    pattern = re.compile(r"^(?i)([a-z]{2,6}-\d+)/.+$")
    return bool(pattern.match(branch_name))

def main():
    current_branch = get_current_branch()
    if not is_branch_name_valid(current_branch):
        print("Branch name '{}' does not follow the 'JIRA-123/some-title' convention.".format(current_branch))
        print("Please rename your branch to include a '/' followed by a title after the JIRA ticket number.")
        sys.exit(1)
    else:
        print("Branch name '{}' is valid.".format(current_branch))
        # Exit with 0 to indicate success
        sys.exit(0)

if __name__ == "__main__":
    main()

