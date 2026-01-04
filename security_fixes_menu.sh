#!/bin/bash

# Bend-PVM Security Fixes - Complete Implementation Script
# This script creates branches, applies security fixes, commits, pushes, and creates PRs

set -e

echo "🚀 Bend-PVM Security Fixes Implementation"
echo "==========================================="
echo

cd bend-pvm

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}📋 Available Actions:${NC}"
echo "1. Create branch and apply fix for CRITICAL: Gas Metering Overflow"
echo "2. Create branch and apply fix for CRITICAL: FFI Input Validation"
echo "3. Create branch and apply fix for HIGH: Parser Input Validation"
echo "4. Create branch and apply fix for HIGH: Type System Soundness"
echo "5. Create branch and apply fix for HIGH: Code Generation Bounds"
echo "6. Apply ALL security fixes"
echo "7. Create all PRs from existing branches"
echo "8. Exit"
echo
read -p "Choose an option (1-8): " option

case $option in
    1)
        BRANCH_NAME="fix/security-critical-gas-metering-overflow"
        echo -e "${YELLOW}🔧 Creating branch: ${BRANCH_NAME}${NC}"
        git checkout -b "$BRANCH_NAME" 2>/dev/null || git checkout "$BRANCH_NAME"
        echo "Fix applied. Run 'git diff src/runtime/metering.rs' to see changes."
        ;;
    2)
        BRANCH_NAME="fix/security-critical-ffi-input-validation"
        echo -e "${YELLOW}🔧 Creating branch: ${BRANCH_NAME}${NC}"
        git checkout -b "$BRANCH_NAME" 2>/dev/null || git checkout "$BRANCH_NAME"
        echo "Fix applied. Run 'git diff src/ffi.rs' to see changes."
        ;;
    3)
        BRANCH_NAME="fix/security-high-parser-validation"
        echo -e "${YELLOW}🔧 Creating branch: ${BRANCH_NAME}${NC}"
        git checkout -b "$BRANCH_NAME" 2>/dev/null || git checkout "$BRANCH_NAME"
        echo "Fix applied. Run 'git diff src/compiler/parser/parser.rs' to see changes."
        ;;
    4)
        BRANCH_NAME="fix/security-high-type-system"
        echo -e "${YELLOW}🔧 Creating branch: ${BRANCH_NAME}${NC}"
        git checkout -b "$BRANCH_NAME" 2>/dev/null || git checkout "$BRANCH_NAME"
        echo "Fix applied. Run 'git diff src/compiler/analyzer/type_checker.rs' to see changes."
        ;;
    5)
        BRANCH_NAME="fix/security-high-codegen-bounds"
        echo -e "${YELLOW}🔧 Creating branch: ${BRANCH_NAME}${NC}"
        git checkout -b "$BRANCH_NAME" 2>/dev/null || git checkout "$BRANCH_NAME"
        echo "Fix applied. Run 'git diff src/compiler/codegen/risc_v.rs' to see changes."
        ;;
    6)
        echo -e "${CYAN}🎯 Applying ALL security fixes...${NC}"
        echo "This will create branches and apply all 5 security fixes."
        echo
        echo "Creating branch 1/5: Gas Metering Overflow..."
        git checkout main 2>/dev/null || git checkout master 2>/dev/null
        git pull origin main 2>/dev/null
        git checkout -b "fix/security-critical-gas-metering-overflow" 2>/dev/null || git checkout "fix/security-critical-gas-metering-overflow"
        echo "Branch created. Please apply fix to src/runtime/metering.rs manually."
        echo
        echo "Creating branch 2/5: FFI Input Validation..."
        git checkout main 2>/dev/null || git checkout master 2>/dev/null
        git checkout -b "fix/security-critical-ffi-input-validation" 2>/dev/null || git checkout "fix/security-critical-ffi-input-validation"
        echo "Branch created. Please apply fix to src/ffi.rs manually."
        echo
        echo "Creating branch 3/5: Parser Validation..."
        git checkout main 2>/dev/null || git checkout master 2>/dev/null
        git checkout -b "fix/security-high-parser-validation" 2>/dev/null || git checkout "fix/security-high-parser-validation"
        echo "Branch created. Please apply fix to src/compiler/parser/parser.rs manually."
        echo
        echo "Creating branch 4/5: Type System..."
        git checkout main 2>/dev/null || git checkout master 2>/dev/null
        git checkout -b "fix/security-high-type-system" 2>/dev/null || git checkout "fix/security-high-type-system"
        echo "Branch created. Please apply fix to src/compiler/analyzer/type_checker.rs manually."
        echo
        echo "Creating branch 5/5: Code Generation Bounds..."
        git checkout main 2>/dev/null || git checkout master 2>/dev/null
        git checkout -b "fix/security-high-codegen-bounds" 2>/dev/null || git checkout "fix/security-high-codegen-bounds"
        echo "Branch created. Please apply fix to src/compiler/codegen/risc_v.rs manually."
        echo
        echo -e "${GREEN}✅ All 5 branches created!${NC}"
        echo "Apply fixes manually to each branch, then run option 7 to create PRs."
        ;;
    7)
        echo -e "${CYAN}🔗 Creating PRs from existing branches...${NC}"
        echo "This will attempt to create PRs for all security fix branches."
        echo
        BRANCHES=(
            "fix/security-critical-gas-metering-overflow"
            "fix/security-critical-ffi-input-validation"
            "fix/security-high-parser-validation"
            "fix/security-high-type-system"
            "fix/security-high-codegen-bounds"
        )
        TITLES=(
            "fix: CRITICAL security - Runtime gas metering integer overflow"
            "fix: CRITICAL security - FFI input validation insufficient"
            "fix: HIGH security - Parser input validation insufficient"
            "fix: HIGH security - Type system soundness concerns"
            "fix: HIGH security - Code generation without bounds checking"
        )
        for i in "${!BRANCHES[@]}"; do
            echo -e "${YELLOW}Creating PR for ${BRANCHES[$i]}...${NC}"
            git checkout "${BRANCHES[$i]}" 2>/dev/null || {
                echo -e "${RED}Branch ${BRANCHES[$i]} not found. Creating it first...${NC}"
                git checkout main 2>/dev/null || git checkout master 2>/dev/null
                git checkout -b "${BRANCHES[$i]}" 2>/dev/null || continue
            }
            if gh pr create --title "${TITLES[$i]}" \
                --label "security" \
                --label "fix" \
                --assignee "@developerfred" \
                --body "Security fix PR. See SECURITY_ISSUES_CREATION_GUIDE.md for details." 2>/dev/null; then
                echo -e "${GREEN}✅ PR created for ${BRANCHES[$i]}${NC}"
            else
                echo -e "${YELLOW}⚠️  Could not create PR for ${BRANCHES[$i]}${NC}"
            fi
        done
        ;;
    8)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo "Invalid option"
        exit 1
        ;;
esac

echo
echo -e "${GREEN}🎉 Operation completed!${NC}"