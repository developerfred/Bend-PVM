//! # Solidity to Bend-PVM Converter
//!
//! This module provides the core conversion logic from Solidity AST
//! to Bend-PVM source code.

use super::ast::*;
use super::{IssueSeverity, MigrationError, MigrationIssue, SolidityMigrator};
use std::collections::HashMap;

/// Converter from Solidity AST to Bend-PVM source code
pub struct SolidityToBendConverter {
    /// Current indentation level
    indent: usize,
    /// Indent string (4 spaces)
    indent_str: String,
    /// Generated output
    output: String,
    /// Type mappings (Solidity type -> Bend type)
    type_mappings: HashMap<String, String>,
    /// Built-in function mappings
    function_mappings: HashMap<String, String>,
    /// Current contract context
    contract_context: Option<String>,
    /// Issues found during conversion
    issues: Vec<MigrationIssue>,
}

impl SolidityToBendConverter {
    /// Create a new converter
    pub fn new() -> Self {
        let mut converter = SolidityToBendConverter {
            indent: 0,
            indent_str: "    ".to_string(),
            output: String::new(),
            type_mappings: HashMap::new(),
            function_mappings: HashMap::new(),
            contract_context: None,
            issues: Vec::new(),
        };
        converter.initialize_mappings();
        converter
    }

    /// Initialize type and function mappings
    fn initialize_mappings(&mut self) {
        // Solidity type to Bend-PVM type mappings
        self.type_mappings
            .insert("uint8".to_string(), "u24".to_string());
        self.type_mappings
            .insert("uint16".to_string(), "u24".to_string());
        self.type_mappings
            .insert("uint24".to_string(), "u24".to_string());
        self.type_mappings
            .insert("uint32".to_string(), "u24".to_string());
        self.type_mappings
            .insert("uint64".to_string(), "u64".to_string());
        self.type_mappings
            .insert("uint128".to_string(), "u128".to_string());
        self.type_mappings
            .insert("uint256".to_string(), "u256".to_string());

        self.type_mappings
            .insert("int8".to_string(), "i24".to_string());
        self.type_mappings
            .insert("int16".to_string(), "i24".to_string());
        self.type_mappings
            .insert("int24".to_string(), "i24".to_string());
        self.type_mappings
            .insert("int32".to_string(), "i24".to_string());
        self.type_mappings
            .insert("int64".to_string(), "i64".to_string());
        self.type_mappings
            .insert("int128".to_string(), "i128".to_string());
        self.type_mappings
            .insert("int256".to_string(), "i256".to_string());

        self.type_mappings
            .insert("bool".to_string(), "Bool".to_string());
        self.type_mappings
            .insert("string".to_string(), "String".to_string());
        self.type_mappings
            .insert("bytes".to_string(), "Bytes".to_string());
        self.type_mappings
            .insert("address".to_string(), "Address".to_string());
        self.type_mappings
            .insert("address payable".to_string(), "Address".to_string());

        // Built-in function mappings
        self.function_mappings
            .insert("require".to_string(), "assert".to_string());
        self.function_mappings
            .insert("revert".to_string(), "assert".to_string());
        self.function_mappings
            .insert("assert".to_string(), "assert".to_string());
        self.function_mappings
            .insert("keccak256".to_string(), "crypto.keccak256".to_string());
        self.function_mappings
            .insert("sha3".to_string(), "crypto.keccak256".to_string());
        self.function_mappings
            .insert("sha256".to_string(), "crypto.sha256".to_string());
        self.function_mappings
            .insert("ripemd160".to_string(), "crypto.ripemd160".to_string());
        self.function_mappings
            .insert("ecrecover".to_string(), "crypto.ecrecover".to_string());
        self.function_mappings
            .insert("block.timestamp".to_string(), "block.timestamp".to_string());
        self.function_mappings
            .insert("block.number".to_string(), "block.height".to_string());
        self.function_mappings
            .insert("msg.sender".to_string(), "ctx.caller".to_string());
        self.function_mappings
            .insert("msg.value".to_string(), "ctx.value".to_string());
        self.function_mappings
            .insert("msg.data".to_string(), "ctx.data".to_string());
        self.function_mappings
            .insert("tx.gasprice".to_string(), "ctx.gas_price".to_string());
        self.function_mappings
            .insert("block.coinbase".to_string(), "block.proposer".to_string());
        self.function_mappings.insert(
            "block.difficulty".to_string(),
            "block.difficulty".to_string(),
        );
        self.function_mappings
            .insert("block.gaslimit".to_string(), "block.gas_limit".to_string());
        self.function_mappings
            .insert("block.chainid".to_string(), "block.chain_id".to_string());
        self.function_mappings
            .insert("gasleft".to_string(), "ctx.gas".to_string());
        self.function_mappings
            .insert("this".to_string(), "ctx.self".to_string());
    }

    /// Convert a Solidity source to Bend-PVM source
    pub fn convert(&mut self, source: &SoliditySource) -> String {
        self.output.clear();
        self.indent = 0;
        self.issues.clear();

        // Add header
        self.add_line("/// Auto-generated Bend-PVM contract from Solidity");
        self.add_line("/// Migration from Solidity smart contracts");
        self.add_line("");
        self.add_line("contract BendContract {");
        self.indent += 1;
        self.add_line("/// Contract context for system calls");
        self.add_line("let ctx: Context");
        self.add_line("");
        self.indent -= 1;
        self.add_line("}");
        self.add_line("");

        // Convert contracts
        for contract in &source.contracts {
            self.convert_contract(contract);
        }

        self.output.clone()
    }

    /// Convert a contract definition
    fn convert_contract(&mut self, contract: &ContractDefinition) {
        // Add contract comment
        self.add_line("");
        self.add_line(&format!("/// Contract: {}", contract.name));

        // Add inheritance info
        if !contract.base_contracts.is_empty() {
            let bases: Vec<String> = contract
                .base_contracts
                .iter()
                .map(|b| b.name.clone())
                .collect();
            self.add_line(&format!("/// Inherits from: {}", bases.join(", ")));
        }

        // Contract definition
        let kind_str = match contract.kind {
            ContractKind::Contract => "contract",
            ContractKind::Interface => "interface",
            ContractKind::Library => "library",
        };

        self.add_line(&format!("{} {} {{", kind_str, contract.name));
        self.indent += 1;
        self.contract_context = Some(contract.name.clone());

        // Convert state variables
        for var in &contract.state_variables {
            self.convert_state_variable(var);
        }

        if !contract.state_variables.is_empty() {
            self.add_line("");
        }

        // Convert events to comments
        for event in &contract.events {
            self.convert_event(event);
        }

        // Convert functions
        for func in &contract.functions {
            self.convert_function(func);
        }

        self.indent -= 1;
        self.add_line("}");
        self.contract_context = None;
    }

    /// Convert a state variable
    fn convert_state_variable(&mut self, var: &StateVariable) {
        let bend_type = self.map_type(&var.type_name);

        // Add documentation
        self.add_line(&format!("/// State variable: {}", var.name));

        // Visibility comment
        let visibility_str = format!("{:?}", var.visibility).to_lowercase();
        self.add_line(&format!("/// Visibility: {}", visibility_str));

        // Variable declaration
        let mut declaration = format!("let {}: {}", var.name, bend_type);

        // Add mutability
        match var.mutability {
            Mutability::Constant => {
                declaration = format!("const {}", declaration);
            }
            Mutability::Immutable => {
                self.add_issue(
                    "Immutable variables require special handling",
                    &format!("{}:{}", var.location.line, var.location.column),
                    IssueSeverity::Partial,
                    Some("Consider using let with init in constructor".to_string()),
                );
            }
            Mutability::Mutable => {}
        }

        // Add initial value if present
        if let Some(value) = &var.value {
            let bend_value = self.convert_expression(value);
            declaration = format!("{} = {}", declaration, bend_value);
        }

        self.add_line(&format!("{};", declaration));
    }

    /// Convert an event
    fn convert_event(&mut self, event: &EventDefinition) {
        self.add_line(&format!("/// Event: {}", event.name));
        let params: Vec<String> = event
            .parameters
            .iter()
            .map(|p| {
                let param_type = self.map_type(&p.type_name);
                format!(
                    "{}: {}",
                    p.name.as_ref().unwrap_or(&String::new()),
                    param_type
                )
            })
            .collect();
        self.add_line(&format!("/// Parameters: {}", params.join(", ")));
        self.add_line(&format!("// emit {}({});", event.name, params.join(", ")));
    }

    /// Convert a function definition
    fn convert_function(&mut self, func: &FunctionDefinition) {
        // Skip special functions that are handled differently
        if func.is_fallback || func.is_receive {
            self.add_issue(
                "Fallback/receive functions need special handling",
                &format!("{}:{}", func.location.line, func.location.column),
                IssueSeverity::Partial,
                Some("Handle via contract entry point configuration".to_string()),
            );
            return;
        }

        // Add documentation
        self.add_line("");
        self.add_line(&format!("/// Function: {}", func.name));

        // Visibility comment
        let visibility_str = format!("{:?}", func.visibility).to_lowercase();
        self.add_line(&format!("/// Visibility: {}", visibility_str));

        // State mutability
        let mutability_str = format!("{:?}", func.state_mutability).to_lowercase();
        self.add_line(&format!("/// Mutability: {}", mutability_str));

        // Function signature
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| self.convert_variable_declaration(p))
            .collect();

        let mut signature = format!("fn {}({})", func.name, params.join(", "));

        // Return type
        if !func.return_parameters.is_empty() {
            let return_types: Vec<String> = func
                .return_parameters
                .iter()
                .map(|p| self.map_type(&p.type_name))
                .collect();
            if return_types.len() == 1 {
                signature = format!("{} -> {}", signature, return_types[0]);
            } else {
                signature = format!("{} -> ({})", signature, return_types.join(", "));
            }
        }

        // Function modifiers
        if !func.modifiers.is_empty() {
            let modifier_names: Vec<String> =
                func.modifiers.iter().map(|m| m.name.clone()).collect();
            self.add_line(&format!("/// Modifiers: {}", modifier_names.join(", ")));
        }

        self.add_line(&format!("{} {{", signature));
        self.indent += 1;

        // Convert function body
        if let Some(body) = &func.body {
            self.convert_block(body);
        } else {
            self.add_line("/// External function - implementation delegated");
        }

        self.indent -= 1;
        self.add_line("}");
    }

    /// Convert a variable declaration to Bend-PVM format
    fn convert_variable_declaration(&self, decl: &VariableDeclaration) -> String {
        let name = decl.name.clone().unwrap_or_else(|| "_".to_string());
        let bend_type = self.map_type(&decl.type_name);
        format!("{}: {}", name, bend_type)
    }

    /// Convert a block of statements
    fn convert_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.convert_statement(stmt);
        }
    }

    /// Convert a statement
    fn convert_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Block(block) => {
                self.add_line("{");
                self.indent += 1;
                self.convert_block(block);
                self.indent -= 1;
                self.add_line("}");
            }
            Statement::VariableDeclaration(var_decl) => {
                let decls: Vec<String> = var_decl
                    .declarations
                    .iter()
                    .filter(|d| d.name.is_some())
                    .map(|d| self.convert_variable_declaration(d))
                    .collect();

                if let Some(init) = &var_decl.initial_value {
                    let init_value = self.convert_expression(init);
                    self.add_line(&format!("let {} = {};", decls.join(", "), init_value));
                } else {
                    self.add_line(&format!("let {};", decls.join(", ")));
                }
            }
            Statement::Expression(expr_stmt) => {
                let expr = self.convert_expression(&expr_stmt.expression);
                self.add_line(&format!("{};", expr));
            }
            Statement::Assignment(assign_stmt) => {
                let left = self.convert_expression(&assign_stmt.assignment.left);
                let right = self.convert_expression(&assign_stmt.assignment.right);

                match assign_stmt.assignment.operator {
                    AssignmentOperator::Assign => {
                        self.add_line(&format!("{} = {};", left, right));
                    }
                    _ => {
                        let op = self.map_assignment_operator(&assign_stmt.assignment.operator);
                        self.add_line(&format!("{} {} {};", left, op, right));
                    }
                }
            }
            Statement::If(if_stmt) => {
                let condition = self.convert_expression(&if_stmt.condition);
                self.add_line(&format!("if {} {{", condition));
                self.indent += 1;
                self.convert_statement(&if_stmt.true_body);
                if let Some(false_body) = &if_stmt.false_body {
                    self.indent -= 1;
                    self.add_line("} else {");
                    self.indent += 1;
                    self.convert_statement(false_body);
                }
                self.indent -= 1;
                self.add_line("}");
            }
            Statement::Return(return_stmt) => {
                if let Some(expr) = &return_stmt.expression {
                    let value = self.convert_expression(expr);
                    self.add_line(&format!("return {};", value));
                } else {
                    self.add_line("return;");
                }
            }
            Statement::Emit(emit_stmt) => {
                let event = self.convert_expression(&emit_stmt.event);
                self.add_line(&format!("emit {};", event));
            }
            Statement::For(_for_stmt) => {
                self.add_line("// for loop - needs manual conversion");
                self.add_line("// Original: for (...) { ... }");
            }
            Statement::While(while_stmt) => {
                let condition = self.convert_expression(&while_stmt.condition);
                self.add_line(&format!("while {} {{", condition));
                self.indent += 1;
                self.convert_statement(&while_stmt.body);
                self.indent -= 1;
                self.add_line("}");
            }
            Statement::Continue(_) => {
                self.add_line("continue;");
            }
            Statement::Break(_) => {
                self.add_line("break;");
            }
            Statement::Revert(revert_stmt) => {
                if let Some(error_call) = &revert_stmt.error_call {
                    let error = self.convert_expression(error_call);
                    self.add_line(&format!("assert(false, {});", error));
                } else {
                    self.add_line("assert(false, \"revert\");");
                }
            }
            Statement::Assembly(assembly) => {
                self.add_line(&format!("// Inline assembly: {}", assembly.operations));
                self.add_issue(
                    "Inline assembly requires manual conversion",
                    &format!("{}:{}", assembly.location.line, assembly.location.column),
                    IssueSeverity::Manual,
                    Some("Rewrite using Bend-PVM inline assembly".to_string()),
                );
            }
            _ => {
                self.add_line("// Statement not fully supported");
            }
        }
    }

    /// Convert an expression
    fn convert_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(identifier) => {
                // Map special identifiers
                match identifier.name.as_str() {
                    "super" => "super",
                    "this" => "self",
                    _ => &identifier.name,
                }
                .to_string()
            }
            Expression::Literal(literal) => {
                if let Some(value) = &literal.value {
                    // Handle string literals
                    if literal
                        .type_name
                        .as_ref()
                        .map(|t| t == "string")
                        .unwrap_or(false)
                    {
                        format!("\"{}\"", value)
                    } else {
                        value.clone()
                    }
                } else {
                    "0".to_string()
                }
            }
            Expression::BinaryOperation(binop) => {
                let left = self.convert_expression(&binop.left);
                let right = self.convert_expression(&binop.right);
                let op = self.map_binary_operator(&binop.operator);
                format!("({} {} {})", left, op, right)
            }
            Expression::UnaryOperation(unop) => {
                let operand = self.convert_expression(&unop.operand);
                let op = self.map_unary_operator(&unop.operator);
                if unop.is_prefix {
                    format!("{}{}", op, operand)
                } else {
                    format!("{}{}", operand, op)
                }
            }
            Expression::FunctionCall(func_call) => {
                let func_expr = self.convert_expression(&func_call.expression);

                // Check if it's a mapped function
                if let Expression::Identifier(id) = &*func_call.expression {
                    if let Some(mapped) = self.function_mappings.get(&id.name) {
                        let args: Vec<String> = func_call
                            .arguments
                            .iter()
                            .map(|a| self.convert_expression(a))
                            .collect();
                        return format!("{}({})", mapped, args.join(", "));
                    }
                }

                let args: Vec<String> = func_call
                    .arguments
                    .iter()
                    .map(|a| self.convert_expression(a))
                    .collect();
                format!("{}({})", func_expr, args.join(", "))
            }
            Expression::MemberAccess(member) => {
                let base = self.convert_expression(&member.expression);
                format!("{}.{}", base, member.member_name)
            }
            Expression::IndexAccess(index) => {
                let base = self.convert_expression(&index.base);
                let idx = self.convert_expression(&index.index);
                format!("{}[{}]", base, idx)
            }
            Expression::Assignment(assign) => {
                let left = self.convert_expression(&assign.left);
                let right = self.convert_expression(&assign.right);
                let op = self.map_assignment_operator(&assign.operator);
                format!("{} {} {}", left, op, right)
            }
            Expression::Conditional(conditional) => {
                let condition = self.convert_expression(&conditional.condition);
                let true_expr = self.convert_expression(&conditional.true_expression);
                let false_expr = self.convert_expression(&conditional.false_expression);
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    condition, true_expr, false_expr
                )
            }
            Expression::Tuple(tuple) => {
                let elements: Vec<String> = tuple
                    .elements
                    .iter()
                    .map(|e| self.convert_expression(e))
                    .collect();
                format!("({})", elements.join(", "))
            }
            Expression::TypeConversion(conv) => {
                let type_str = self.map_type(&conv.type_name);
                let expr = self.convert_expression(&conv.expression);
                format!("{}({})", type_str, expr)
            }
            _ => "unknown_expression".to_string(),
        }
    }

    /// Map a Solidity type to Bend-PVM type
    fn map_type(&self, type_name: &TypeName) -> String {
        match type_name {
            TypeName::Elementary(elem) => self
                .type_mappings
                .get(&elem.name)
                .cloned()
                .unwrap_or_else(|| elem.name.clone()),
            TypeName::UserDefined(user) => {
                // Handle type arguments
                if user.type_arguments.is_empty() {
                    user.name.clone()
                } else {
                    let args: Vec<String> = user
                        .type_arguments
                        .iter()
                        .map(|t| self.map_type(t))
                        .collect();
                    format!("{}<{}>", user.name, args.join(", "))
                }
            }
            TypeName::Array(array) => {
                let base_type = self.map_type(&array.base_type);
                if let Some(len) = &array.length {
                    format!("[{}; {}]", base_type, self.convert_expression(len))
                } else {
                    format!("List<{}>", base_type)
                }
            }
            TypeName::Mapping(mapping) => {
                let key_type = self.map_type(&mapping.key_type);
                let value_type = self.map_type(&mapping.value_type);
                format!("Map<{}, {}>", key_type, value_type)
            }
            TypeName::Function(func) => {
                let params: Vec<String> = func
                    .parameter_types
                    .iter()
                    .map(|t| self.map_type(t))
                    .collect();
                let returns: Vec<String> =
                    func.return_types.iter().map(|t| self.map_type(t)).collect();
                format!("fn({}) -> {}", params.join(", "), returns.join(", "))
            }
        }
    }

    /// Map binary operator
    fn map_binary_operator(&self, op: &BinaryOperator) -> String {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Sub => "-",
            BinaryOperator::Mul => "*",
            BinaryOperator::Div => "/",
            BinaryOperator::Mod => "%",
            BinaryOperator::Pow => "**",
            BinaryOperator::BitAnd => "&",
            BinaryOperator::BitOr => "|",
            BinaryOperator::BitXor => "^",
            BinaryOperator::BitShiftLeft => "<<",
            BinaryOperator::BitShiftRight => ">>",
            BinaryOperator::LogicalAnd => "&&",
            BinaryOperator::LogicalOr => "||",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::Less => "<",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::Greater => ">",
            BinaryOperator::GreaterEqual => ">=",
        }
        .to_string()
    }

    /// Map unary operator
    fn map_unary_operator(&self, op: &UnaryOperator) -> String {
        match op {
            UnaryOperator::Not => "!",
            UnaryOperator::Neg => "-",
            UnaryOperator::BitNot => "~",
            UnaryOperator::Inc => "++",
            UnaryOperator::Dec => "--",
        }
        .to_string()
    }

    /// Map assignment operator
    fn map_assignment_operator(&self, op: &AssignmentOperator) -> String {
        match op {
            AssignmentOperator::Assign => "=",
            AssignmentOperator::AddAssign => "+=",
            AssignmentOperator::SubAssign => "-=",
            AssignmentOperator::MulAssign => "*=",
            AssignmentOperator::DivAssign => "/=",
            AssignmentOperator::ModAssign => "%=",
            AssignmentOperator::BitAndAssign => "&=",
            AssignmentOperator::BitOrAssign => "|=",
            AssignmentOperator::BitXorAssign => "^=",
            AssignmentOperator::BitShiftLeftAssign => "<<=",
            AssignmentOperator::BitShiftRightAssign => ">>=",
        }
        .to_string()
    }

    /// Add a line to the output
    fn add_line(&mut self, line: &str) {
        let indented = format!("{}{}", self.indent_str.repeat(self.indent), line);
        self.output.push_str(&indented);
        self.output.push('\n');
    }

    /// Add a migration issue
    fn add_issue(
        &mut self,
        description: &str,
        location: &str,
        severity: IssueSeverity,
        suggestion: Option<String>,
    ) {
        self.issues.push(MigrationIssue {
            description: description.to_string(),
            source_location: location.to_string(),
            severity,
            suggestion,
        });
    }

    /// Get conversion issues
    pub fn get_issues(&self) -> &[MigrationIssue] {
        &self.issues
    }
}

impl Default for SolidityToBendConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for SolidityMigrator to add conversion support
impl SolidityMigrator {
    /// Convert a Solidity source file to Bend-PVM source
    pub fn convert_solidity(&mut self, source: &SoliditySource) -> Result<String, MigrationError> {
        let mut converter = SolidityToBendConverter::new();
        let result = converter.convert(source);

        // Log issues
        for issue in converter.get_issues() {
            self.stats.issues.push(issue.clone());
        }

        self.stats.contracts_processed += 1;
        self.stats.functions_translated += source
            .contracts
            .iter()
            .map(|c| c.functions.len())
            .sum::<usize>();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_creation() {
        let converter = SolidityToBendConverter::new();
        assert!(!converter.type_mappings.is_empty());
        assert!(!converter.function_mappings.is_empty());
    }

    #[test]
    fn test_type_mapping() {
        let converter = SolidityToBendConverter::new();
        assert_eq!(
            converter.type_mappings.get("uint256"),
            Some(&"u256".to_string())
        );
        assert_eq!(
            converter.type_mappings.get("bool"),
            Some(&"Bool".to_string())
        );
    }

    #[test]
    fn test_function_mapping() {
        let converter = SolidityToBendConverter::new();
        assert_eq!(
            converter.function_mappings.get("require"),
            Some(&"assert".to_string())
        );
        assert_eq!(
            converter.function_mappings.get("keccak256"),
            Some(&"crypto.keccak256".to_string())
        );
    }
}
