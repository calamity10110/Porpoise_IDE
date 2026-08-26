#!/bin/bash
# Bump version in all Cargo.toml files
# Usage: ./scripts/bump-version.sh 0.2.0

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

VERSION=$1
echo "Bumping version to $VERSION..."

# Update workspace root
sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# Update all crate Cargo.toml files
find crates -name "Cargo.toml" -exec sed -i "s/^version = .*/version = \"$VERSION\"/" {} \;

echo "Version bumped to $VERSION in all Cargo.toml files."
echo "Run 'cargo build --workspace' to verify."
