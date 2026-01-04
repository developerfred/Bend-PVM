#!/bin/bash

# Bend-PVM Issues Analysis Script
# Fetches and analyzes all open issues from GitHub

echo "🔍 Bend-PVM Issues Analysis"
echo "=========================="
echo

cd bend-pvm

# Issues to analyze
ISSUES=("10" "9" "8" "7" "6" "5" "4")

echo "📋 Fetching open issues from GitHub..."
echo

for issue in "${ISSUES[@]}"; do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${CYAN}Issue #$issue${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Fetch issue details
    if gh issue view "$issue" --json title,state,labels,body,author,createdAt,assignees 2>/dev/null; then
        echo
    else
        echo -e "${YELLOW}Could not fetch issue #$issue from GitHub${NC}"
        echo "This might be because:"
        echo "  - The issue number is incorrect"
        echo "  - GitHub CLI is not authenticated"
        echo "  - The repository is not accessible"
        echo
    fi
    
    # Check if there are any related files in the codebase
    case $issue in
        10)
            echo -e "${BLUE}Related implementation files:${NC}"
            ls -la src/runtime/storage.rs 2>/dev/null && echo "  ✅ src/runtime/storage.rs exists" || echo "  ❌ src/runtime/storage.rs not found"
            ;;
        9)
            echo -e "${BLUE}Related implementation files:${NC}"
            ls -la src/runtime/metering.rs 2>/dev/null && echo "  ✅ src/runtime/metering.rs exists" || echo "  ❌ src/runtime/metering.rs not found"
            ;;
        8)
            echo -e "${BLUE}Related implementation files:${NC}"
            ls -la src/compiler/polkavm/ 2>/dev/null && echo "  ✅ src/compiler/polkavm/ exists" || echo "  ❌ src/compiler/polkavm/ not found"
            ls -la src/compiler/codegen/risc_v.rs 2>/dev/null && echo "  ✅ src/compiler/codegen/risc_v.rs exists" || echo "  ❌ src/compiler/codegen/risc_v.rs not found"
            ;;
        7)
            echo -e "${BLUE}Related implementation files:${NC}"
            ls -la src/compiler/codegen/risc_v.rs 2>/dev/null && echo "  ✅ src/compiler/codegen/risc_v.rs exists" || echo "  ❌ src/compiler/codegen/risc_v.rs not found"
            ls -la src/compiler/codegen/ir.rs 2>/dev/null && echo "  ✅ src/compiler/codegen/ir.rs exists" || echo "  ❌ src/compiler/codegen/ir.rs not found"
            ;;
        6)
            echo -e "${BLUE}Related implementation files:${NC}"
            ls -la src/compiler/parser/parser.rs 2>/dev/null && echo "  ✅ src/compiler/parser/parser.rs exists" || echo "  ❌ src/compiler/parser/parser.rs not found"
            ls -la src/compiler/parser/ast.rs 2>/dev/null && echo "  ✅ src/compiler/parser/ast.rs exists" || echo "  ❌ src/compiler/parser/ast.rs not found"
            ;;
        5)
            echo -e "${BLUE}Related implementation files:${NC}"
            ls -la src/compiler/lexer/lexer.rs 2>/dev/null && echo "  ✅ src/compiler/lexer/lexer.rs exists" || echo "  ❌ src/compiler/lexer/lexer.rs not found"
            ls -la src/compiler/lexer/token.rs 2>/dev/null && echo "  ✅ src/compiler/lexer/token.rs exists" || echo "  ❌ src/compiler/lexer/token.rs not found"
            ;;
        4)
            echo -e "${BLUE}Related implementation files:${NC}"
            ls -la src/compiler/module/ 2>/dev/null && echo "  ✅ src/compiler/module/ exists" || echo "  ❌ src/compiler/module/ not found"
            ;;
    esac
    
    echo
done

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}📊 Issues Summary${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Check which files exist
echo "🔍 Current Implementation Status:"
echo

# Check RUNTIME files
echo "RUNTIME Issues:"
echo "  Issue #10 (Storage System): $([ -f src/runtime/storage.rs ] && echo '✅ Implemented' || echo '❌ Not found')"
echo "  Issue #9 (Gas Metering): $([ -f src/runtime/metering.rs ] && echo '✅ Implemented' || echo '❌ Not found')"
echo "  Issue #8 (PolkaVM Bridge): $([ -d src/compiler/polkavm/ ] && echo '✅ Implemented' || echo '❌ Not found')"
echo

# Check COMPILER files
echo "COMPILER Issues:"
echo "  Issue #7 (RISC-V Codegen): $([ -f src/compiler/codegen/risc_v.rs ] && echo '✅ Implemented' || echo '❌ Not found')"
echo "  Issue #6 (Parser): $([ -f src/compiler/parser/parser.rs ] && echo '✅ Implemented' || echo '❌ Not found')"
echo "  Issue #5 (Lexer): $([ -f src/compiler/lexer/lexer.rs ] && echo '✅ Implemented' || echo '❌ Not found')"
echo

# Check CORE files
echo "CORE Issues Issue #4 (:"
echo " Module System): $([ -d src/compiler/module/ ] && echo '✅ Implemented' || echo '❌ Not found')"
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}📝 Analysis Notes${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Based on the file system check:"
echo "  - Most core compiler files appear to exist"
echo "  - Implementation may be partial or incomplete"
echo "  - Issues may need to be updated to reflect current state"
echo
echo "To get accurate issue information from GitHub:"
echo "  1. Ensure GitHub CLI is authenticated: gh auth login"
echo "  2. Check issue status: gh issue list"
echo "  3. View specific issue: gh issue view <number>"
echo
echo "To update issue status on GitHub:"
echo "  1. Update issue: gh issue edit <number> --state closed"
echo "  2. Add comment: gh issue comment <number> --body 'Update'"
echo
echo -e "${GREEN}✅ Analysis complete!${NC}"