#!/bin/bash
# Generate changelog from git log
# Usage: ./scripts/changelog.sh [since_tag]

set -e

SINCE=${1:-$(git describe --tags --abbrev=0 2>/dev/null || echo "")}

if [ -z "$SINCE" ]; then
    echo "# Changelog"
    echo ""
    git log --pretty=format:"- %s (%h)" --no-merges
else
    echo "# Changelog (since $SINCE)"
    echo ""
    git log "$SINCE..HEAD" --pretty=format:"- %s (%h)" --no-merges
fi
