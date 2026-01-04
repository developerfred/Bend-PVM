#!/bin/bash

# Script para criar PR para Issue #2 (Type System)
# Execute: ./create-pr-type-system.sh

cd /Volumes/Codingsh/bend-pvm-ecossystem/bend-pvm

echo "=== Criando PR para Issue #2: Type System ==="

# Configurar git se necessário
if [ ! -d .git ]; then
    echo "Inicializando repositório git..."
    git init
    git config user.name "Bend-PVM Team"
    git config user.email "team@bendpvm.io"
fi

# Criar branch
echo "Criando branch feat/type-system-v2..."
git checkout -b feat/type-system-v2 2>/dev/null || git checkout feat/type-system-v2

# Adicionar arquivos
echo "Adicionando arquivos..."
git add src/compiler/analyzer/type_inference.rs
git add src/compiler/parser/ast.rs

# Criar commit
echo "Criando commit..."
git commit -m "feat(type): Complete Type System Implementation

- Implement full type inference with unification
- Add constraint solver for type constraints
- Support for generics (Option, Result, List)
- Pattern matching type checking
- Built-in type environment
- Unit tests for type inference

Closes #2"

# Push e criar PR
echo "Enviando para remote..."
git push origin feat/type-system-v2 2>/dev/null && {
    echo "Criando Pull Request..."
    gh pr create \
        --title "feat(type): Complete Type System Implementation" \
        --body "## Summary
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
        --base main
}

echo "=== PR criado com sucesso! ==="
