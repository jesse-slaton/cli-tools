#!/bin/bash
#
# Install Git hooks for the cli-tools repository
# This script copies hook templates from hooks/ to .git/hooks/

set -e

# ANSI color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Installing Git hooks for cli-tools repository...${NC}"
echo ""

# Get the repository root directory
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/hooks"
GIT_HOOKS_DIR="$REPO_ROOT/.git/hooks"

# Check if we're in a git repository
if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo -e "${YELLOW}Error: Not in a git repository or .git/hooks directory not found${NC}"
    exit 1
fi

# Check if hooks directory exists
if [ ! -d "$HOOKS_DIR" ]; then
    echo -e "${YELLOW}Error: hooks/ directory not found${NC}"
    exit 1
fi

# Install pre-commit hook
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    echo -e "${BLUE}→${NC} Installing pre-commit hook..."
    cp "$HOOKS_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
    chmod +x "$GIT_HOOKS_DIR/pre-commit"
    echo -e "${GREEN}  ✓ pre-commit hook installed${NC}"
else
    echo -e "${YELLOW}  ! pre-commit hook template not found${NC}"
fi

echo ""
echo -e "${GREEN}✓ Git hooks installation complete!${NC}"
echo ""
echo "The following checks will run before each commit:"
echo "  • Code formatting (cargo fmt)"
echo "  • Linting (cargo clippy)"
echo "  • Build verification (cargo check)"
echo "  • Unit tests (cargo test)"
echo "  • Documentation build (cargo doc)"
echo "  • Security audit (cargo audit, if installed)"
echo ""
echo -e "${BLUE}Note:${NC} You can bypass hooks with: ${YELLOW}git commit --no-verify${NC}"
echo ""
