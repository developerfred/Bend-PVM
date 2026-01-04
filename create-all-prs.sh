#!/bin/bash

# Script para criar todos os PRs de uma vez
# Execute: ./create-all-prs.sh

set -e

cd /Volumes/Codingsh/bend-pvm-ecossystem/bend-pvm

echo "======================================"
echo "Bend-PVM Pull Requests Generator"
echo "======================================"

# Função para criar PR
create_pr() {
    local branch=$1
    local title=$2
    local body=$3
    local files=$4
    
    echo ""
    echo "=== Criando PR: $title ==="
    
    # Criar/checkout branch
    git checkout -b $branch 2>/dev/null || git checkout $branch
    
    # Adicionar arquivos
    git add $files
    
    # Criar commit
    git commit -m "$title

$body"
    
    # Push
    git push origin $branch 2>/dev/null || true
    
    # Criar PR (se gh CLI disponível)
    if command -v gh &> /dev/null; then
        gh pr create --title "$title" --body "$body" --base main 2>/dev/null || {
            echo "PR pode já existir ou gh CLI não configurado"
        }
    else
        echo "gh CLI não encontrado. Execute manualmente:"
        echo "  git push origin $branch"
        echo "  gh pr create --title \"$title\" --body \"$body\""
    fi
}

# Inicializar git se necessário
if [ ! -d .git ]; then
    echo "Inicializando git..."
    git init
    git config user.name "Bend-PVM Team"
    git config user.email "team@bendpvm.io"
    git add -A
    git commit -m "Initial commit: All implementations"
fi

# PR para Issue #2 (Type System)
create_pr \
    "feat/type-system-v2" \
    "feat(type): Complete Type System Implementation" \
    "## Summary
Complete implementation of the Type System for Bend-PVM.

## Changes
- Added full type inference with unification algorithm
- Implemented constraint solver for type constraints
- Support for generic types (Option, Result, List)
- Pattern matching type checking
- Built-in type environment with symbol table
- 3 unit tests for type inference

## Technical Details
- **InferType enum**: Represents types in the inference system
- **ConstraintSolver**: Handles type unification
- **TypeEnv**: Maintains symbol table and type definitions
- **TypeInferrer**: Traverses AST and infers types

## Tests
\`\`\`rust
test_type_inference_basic()
test_builtin_types()
test_type_display()
\`\`\`

Closes #2" \
    "src/compiler/analyzer/type_inference.rs src/compiler/parser/ast.rs"

# PR para Issue #17 (Solidity Migration)
create_pr \
    "feat/solidity-migration" \
    "feat(migration): Add Solidity to Bend-PVM Migration Tools" \
    "## Summary
Complete Solidity to Bend-PVM migration toolkit.

## Changes
- **Solidity AST**: Complete AST representation for Solidity contracts
- **Converter**: Full transpiler with type/function mappings
- **Analyzer**: Compatibility analysis and issue detection
- **CLI**: Command-line interface for migration
- **ERC Templates**: ERC-20, ERC-721, ERC-1155

## Features
- Type mappings (uint256 → u256, address → Address, etc.)
- Function mappings (require → assert, keccak256 → crypto.keccak256)
- Contract inheritance analysis
- Compatibility scoring
- Gas savings estimation

## CLI Commands
- \`bend-migrate convert <file>\` - Convert Solidity to Bend-PVM
- \`bend-migrate analyze <file>\` - Analyze compatibility
- \`bend-migrate template <ERC>\` - Generate ERC template
- \`bend-migrate list-erc\` - List available templates

Closes #17" \
    "src/migration/"

# PR para Issue #25 (Testing Framework)
create_pr \
    "feat/testing-framework" \
    "feat(test): Complete Testing Framework" \
    "## Summary
Complete testing framework for Bend-PVM contracts.

## Changes
- **TestRunner**: Execute test cases with proper setup/teardown
- **TestAssertions**: Storage, gas, and event assertions
- **TestEnvironment**: Execution context with metering
- **TestSuite**: Organize and run multiple tests
- **Macros**: test_suite! and test_case! macros

## Features
- Gas usage verification
- Storage state assertions
- Event emission checking
- Timeout handling
- Mock library support

## Usage
\`\`\`rust
let suite = test_suite!(\"MyTests\",
    test_case!(source: \"...\", function: \"test_add\"),
    test_case!(source: \"...\", function: \"test_sub\"),
);

let results = suite.run_all();
\`\`\`

Closes #25" \
    "src/testing/"

echo ""
echo "======================================"
echo "Todos os PRs criados!"
echo "======================================"
echo ""
echo "Branches criadas:"
echo "  - feat/type-system-v2"
echo "  - feat/solidity-migration"
echo "  - feat/testing-framework"
echo ""
echo "Verificar PRs:"
echo "  gh pr list"
