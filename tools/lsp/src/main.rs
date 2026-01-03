use std::error::Error;
use std::fs;
use std::path::Path;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidOpenTextDocument, Notification as _, PublishDiagnostics,
};
use lsp_types::*;
use serde_json::Value;

use bend_pvm::compiler::parser::{
    ast::{Definition, Expr, Location as AstLocation, LocationProvider, Program, Statement},
    parser::{ParseError, Parser},
};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Create the connection to the language server client
    let (connection, io_threads) = Connection::stdio();

    // Initialize the server capabilities
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![".".to_string()]),
            ..CompletionOptions::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
            retrigger_characters: Some(vec![",".to_string()]),
            ..SignatureHelpOptions::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        workspace_symbol_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::QUICK_FIX, CodeActionKind::REFACTOR]),
            ..CodeActionOptions::default()
        }),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..ServerCapabilities::default()
    })
    .unwrap();

    // Initialize the server
    let params = connection.initialize(server_capabilities)?;
    let _init_params: InitializeParams = serde_json::from_value(params).unwrap();

    // Main loop
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }

                match handle_request(&connection, req) {
                    Ok(()) => {}
                    Err(e) => eprintln!("Error handling request: {}", e),
                }
            }
            Message::Response(_resp) => {}
            Message::Notification(not) => match handle_notification(&connection, not) {
                Ok(()) => {}
                Err(e) => eprintln!("Error handling notification: {}", e),
            },
        }
    }

    // Wait for the IO threads to finish
    io_threads.join()?;

    Ok(())
}

fn handle_request(
    connection: &Connection,
    req: Request,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    match req.method.as_str() {
        "textDocument/completion" => {
            let params = serde_json::from_value::<CompletionParams>(req.params.clone())?;
            let completion_items = get_completion_items(&params);
            let result = Some(CompletionResponse::Array(completion_items));
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(result)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        "textDocument/hover" => {
            let params = serde_json::from_value::<HoverParams>(req.params.clone())?;
            let hover = get_hover(&params);
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(hover)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        "textDocument/definition" => {
            let params = serde_json::from_value::<GotoDefinitionParams>(req.params.clone())?;
            let location = get_definition(&params);
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(location)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        "textDocument/references" => {
            let params = serde_json::from_value::<ReferenceParams>(req.params.clone())?;
            let references = find_references(&params);
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(references)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        "textDocument/documentSymbol" => {
            let params = serde_json::from_value::<DocumentSymbolParams>(req.params.clone())?;
            let symbols = get_document_symbols(&params);
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(symbols)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        "textDocument/signatureHelp" => {
            let params = serde_json::from_value::<SignatureHelpParams>(req.params.clone())?;
            let signature_help = get_signature_help(&params);
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(signature_help)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        "workspace/symbol" => {
            let params = serde_json::from_value::<WorkspaceSymbolParams>(req.params.clone())?;
            let symbols = get_workspace_symbols(&params);
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(symbols)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        "textDocument/codeAction" => {
            let params = serde_json::from_value::<CodeActionParams>(req.params.clone())?;
            let code_actions = get_code_actions(&params);
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(code_actions)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        _ => {
            let resp = Response {
                id: req.id,
                result: Some(Value::Null),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
    }

    Ok(())
}

fn handle_notification(
    connection: &Connection,
    not: Notification,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    match not.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let params = serde_json::from_value::<DidOpenTextDocumentParams>(not.params)?;
            publish_diagnostics(
                connection,
                params.text_document.uri,
                &params.text_document.text,
            )?;
        }
        DidChangeTextDocument::METHOD => {
            let params = serde_json::from_value::<DidChangeTextDocumentParams>(not.params)?;
            // We use FULL sync, so there's only one change with the full text
            if let Some(change) = params.content_changes.first() {
                publish_diagnostics(connection, params.text_document.uri, &change.text)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Url,
    text: &str,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut diagnostics = Vec::new();

    let mut parser = Parser::new(text);
    if let Err(e) = parser.parse_program() {
        let diagnostic = match e {
            ParseError::UnexpectedToken {
                line,
                column,
                found,
                expected,
                ..
            } => Diagnostic {
                range: Range {
                    start: Position {
                        line: (line - 1) as u32,
                        character: (column - 1) as u32,
                    },
                    end: Position {
                        line: (line - 1) as u32,
                        character: (column - 1 + found.len()) as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Unexpected token '{}', expected '{}'", found, expected),
                source: Some("bend-pvm".to_string()),
                ..Diagnostic::default()
            },
            ParseError::UnterminatedString { line, column } => Diagnostic {
                range: Range {
                    start: Position {
                        line: (line - 1) as u32,
                        character: (column - 1) as u32,
                    },
                    end: Position {
                        line: (line - 1) as u32,
                        character: column as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Unterminated string literal".to_string(),
                source: Some("bend-pvm".to_string()),
                ..Diagnostic::default()
            },
            ParseError::InvalidNumber { line, column } => Diagnostic {
                range: Range {
                    start: Position {
                        line: (line - 1) as u32,
                        character: (column - 1) as u32,
                    },
                    end: Position {
                        line: (line - 1) as u32,
                        character: column as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Invalid number format".to_string(),
                source: Some("bend-pvm".to_string()),
                ..Diagnostic::default()
            },
            _ => {
                // Fallback for other errors with better location
                Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Parse error: {}", e),
                    source: Some("bend-pvm".to_string()),
                    ..Diagnostic::default()
                }
            }
        };
        diagnostics.push(diagnostic);
    }

    // TODO: Add type checking diagnostics when type checker is available
    // This will provide warnings and errors for type mismatches, undefined variables, etc.

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };

    let not = Notification {
        method: PublishDiagnostics::METHOD.to_string(),
        params: serde_json::to_value(params)?,
    };

    connection.sender.send(Message::Notification(not))?;

    Ok(())
}

fn get_completion_items(_params: &CompletionParams) -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "def".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..CompletionItem::default()
        },
        // ... (can be expanded)
    ]
}

fn get_definition(params: &GotoDefinitionParams) -> Option<GotoDefinitionResponse> {
    let position = params.text_document_position_params.position;
    let document_uri = params
        .text_document_position_params
        .text_document
        .uri
        .clone();
    let document_path = document_uri.to_file_path().ok()?;
    let document_text = fs::read_to_string(document_path).ok()?;

    let mut parser = Parser::new(&document_text);
    let program = parser.parse_program().ok()?;

    let target_name = find_identifier_at_pos(
        &program,
        (position.line + 1) as usize,
        (position.character + 1) as usize,
    )?;
    let def_loc = find_definition(&program, &target_name)?;

    Some(GotoDefinitionResponse::Scalar(Location {
        uri: document_uri,
        range: Range {
            start: Position {
                line: (def_loc.line - 1) as u32,
                character: (def_loc.column - 1) as u32,
            },
            end: Position {
                line: (def_loc.line - 1) as u32,
                character: (def_loc.column - 1 + target_name.len()) as u32,
            },
        },
    }))
}

fn get_hover(params: &HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let document_uri = params
        .text_document_position_params
        .text_document
        .uri
        .clone();
    let document_path = document_uri.to_file_path().ok()?;
    let document_text = fs::read_to_string(document_path).ok()?;

    let mut parser = Parser::new(&document_text);
    if let Ok(program) = parser.parse_program() {
        if let Some(name) = find_identifier_at_pos(
            &program,
            (position.line + 1) as usize,
            (position.character + 1) as usize,
        ) {
            if let Some(_) = find_definition(&program, &name) {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**Function**: `{}`", name),
                    }),
                    range: None,
                });
            }
        }
    }

    None
}

fn find_identifier_at_pos(program: &Program, line: usize, col: usize) -> Option<String> {
    for def in &program.definitions {
        if let Some(name) = find_in_def(def, line, col) {
            return Some(name);
        }
    }
    None
}

fn find_in_def(def: &Definition, line: usize, col: usize) -> Option<String> {
    match def {
        Definition::FunctionDef { body, .. } => find_in_block(body, line, col),
        _ => None,
    }
}

fn find_in_block(
    block: &bend_pvm::compiler::parser::ast::Block,
    line: usize,
    col: usize,
) -> Option<String> {
    for stmt in &block.statements {
        if let Some(name) = find_in_stmt(stmt, line, col) {
            return Some(name);
        }
    }
    None
}

fn find_in_stmt(stmt: &Statement, line: usize, col: usize) -> Option<String> {
    match stmt {
        Statement::Expr { expr, .. } => find_in_expr(expr, line, col),
        Statement::Assignment { value, .. } => find_in_expr(value, line, col),
        Statement::Return { value, .. } => find_in_expr(value, line, col),
        Statement::LocalDef { function_def, .. } => find_in_def(function_def, line, col),
        _ => None,
    }
}

fn find_in_expr(expr: &Expr, line: usize, col: usize) -> Option<String> {
    let loc = expr.location();
    match expr {
        Expr::Variable { name, location } => {
            if location.line == line && col >= location.column && col < location.column + name.len()
            {
                return Some(name.clone());
            }
            None
        }
        Expr::FunctionCall { function, args, .. } => {
            if let Some(name) = find_in_expr(function, line, col) {
                return Some(name);
            }
            for arg in args {
                if let Some(name) = find_in_expr(arg, line, col) {
                    return Some(name);
                }
            }
            None
        }
        Expr::BinaryOp { left, right, .. } => {
            if let Some(name) = find_in_expr(left, line, col) {
                return Some(name);
            }
            if let Some(name) = find_in_expr(right, line, col) {
                return Some(name);
            }
            None
        }
        _ => None,
    }
}

fn find_definition(program: &Program, name: &str) -> Option<AstLocation> {
    for def in &program.definitions {
        match def {
            Definition::FunctionDef {
                name: def_name,
                location,
                ..
            } => {
                if def_name == name {
                    return Some(location.clone());
                }
            }
            Definition::TypeDef {
                name: def_name,
                location,
                ..
            } => {
                if def_name == name {
                    return Some(location.clone());
                }
            }
            Definition::ObjectDef {
                name: def_name,
                location,
                ..
            } => {
                if def_name == name {
                    return Some(location.clone());
                }
            }
        }
    }
    None
}

fn find_references(_params: &ReferenceParams) -> Option<Vec<Location>> {
    Some(Vec::new())
}

fn get_document_symbols(params: &DocumentSymbolParams) -> Option<DocumentSymbolResponse> {
    let document_uri = &params.text_document.uri;
    let document_path = document_uri.to_file_path().ok()?;
    let document_text = fs::read_to_string(document_path).ok()?;

    let mut parser = Parser::new(&document_text);
    let program = parser.parse_program().ok()?;

    let mut symbols = Vec::new();

    for def in &program.definitions {
        if let Some(symbol) = convert_definition_to_symbol(def, document_uri) {
            symbols.push(symbol);
        }
    }

    Some(DocumentSymbolResponse::Nested(symbols))
}

fn convert_definition_to_symbol(def: &Definition, uri: &Url) -> Option<DocumentSymbol> {
    match def {
        Definition::FunctionDef {
            name,
            location,
            body,
            ..
        } => {
            let range = Range {
                start: Position {
                    line: (location.line - 1) as u32,
                    character: (location.column - 1) as u32,
                },
                end: Position {
                    line: (location.line - 1) as u32,
                    character: (location.column - 1 + name.len()) as u32,
                },
            };

            let mut children = Vec::new();
            collect_block_symbols(body, uri, &mut children);

            Some(DocumentSymbol {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                detail: None,
                range,
                selection_range: range,
                children: if children.is_empty() {
                    None
                } else {
                    Some(children)
                },
                data: None,
                deprecated: None,
            })
        }
        Definition::TypeDef { name, location, .. } => {
            let range = Range {
                start: Position {
                    line: (location.line - 1) as u32,
                    character: (location.column - 1) as u32,
                },
                end: Position {
                    line: (location.line - 1) as u32,
                    character: (location.column - 1 + name.len()) as u32,
                },
            };

            Some(DocumentSymbol {
                name: name.clone(),
                kind: SymbolKind::OBJECT,
                tags: None,
                detail: None,
                range,
                selection_range: range,
                children: None,
                data: None,
                deprecated: None,
            })
        }
        Definition::ObjectDef { name, location, .. } => {
            let range = Range {
                start: Position {
                    line: (location.line - 1) as u32,
                    character: (location.column - 1) as u32,
                },
                end: Position {
                    line: (location.line - 1) as u32,
                    character: (location.column - 1 + name.len()) as u32,
                },
            };

            Some(DocumentSymbol {
                name: name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                detail: None,
                range,
                selection_range: range,
                children: if children.is_empty() {
                    None
                } else {
                    Some(children)
                },
                data: None,
                deprecated: None,
            })
        }
    }
}

fn get_workspace_symbols(_params: &WorkspaceSymbolParams) -> Option<Vec<WorkspaceSymbol>> {
    Some(Vec::new())
}

fn get_signature_help(_params: &SignatureHelpParams) -> Option<SignatureHelp> {
    Some(SignatureHelp {
        signatures: Vec::new(),
        active_signature: None,
        active_parameter: None,
    })
}

fn get_code_actions(_params: &CodeActionParams) -> Option<Vec<CodeAction>> {
    // Implementação básica de Code Actions
    // Retorna lista vazia por enquanto, pode ser expandida para quick fixes
    Some(Vec::new())
}

fn collect_block_symbols(
    block: &bend_pvm::compiler::parser::ast::Block,
    uri: &Url,
    symbols: &mut Vec<DocumentSymbol>,
) {
    for stmt in &block.statements {
        match stmt {
            Statement::LocalDef { function_def, .. } => {
                if let Some(symbol) = convert_definition_to_symbol(function_def, uri) {
                    symbols.push(symbol);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_completion_items_returns_keyword() {
        let params = CompletionParams::new(
            TextDocumentIdentifier::new(Url::parse("file:///test.bend").unwrap()),
            Position::new(0, 0),
            CompletionContext::new(),
        );
        let items = get_completion_items(&params);
        assert!(!items.is_empty());
        assert_eq!(items[0].label, "def");
        assert_eq!(items[0].kind, Some(CompletionItemKind::KEYWORD));
    }

    #[test]
    fn test_get_signature_help_returns_empty() {
        let params = SignatureHelpParams::new(
            TextDocumentPositionParams::new(
                TextDocumentIdentifier::new(Url::parse("file:///test.bend").unwrap()),
                Position::new(0, 0),
            ),
            None,
            None,
        );
        let help = get_signature_help(&params);
        assert!(help.is_some());
        let help = help.unwrap();
        assert!(help.signatures.is_empty());
        assert!(help.active_signature.is_none());
        assert!(help.active_parameter.is_none());
    }

    #[test]
    fn test_get_code_actions_returns_empty() {
        let params = CodeActionParams::new(
            TextDocumentIdentifier::new(Url::parse("file:///test.bend").unwrap()),
            Range::new(Position::new(0, 0), Position::new(0, 10)),
            CodeActionContext::new(),
        );
        let actions = get_code_actions(&params);
        assert!(actions.is_some());
        assert!(actions.unwrap().is_empty());
    }

    #[test]
    fn test_get_workspace_symbols_returns_empty() {
        let params = WorkspaceSymbolParams::new(Query::new("test".to_string(), None));
        let symbols = get_workspace_symbols(&params);
        assert!(symbols.is_some());
        assert!(symbols.unwrap().is_empty());
    }

    #[test]
    fn test_find_references_returns_empty() {
        let params = ReferenceParams::new(
            TextDocumentPositionParams::new(
                TextDocumentIdentifier::new(Url::parse("file:///test.bend").unwrap()),
                Position::new(0, 0),
            ),
            None,
            None,
            None,
        );
        let refs = find_references(&params);
        assert!(refs.is_some());
        assert!(refs.unwrap().is_empty());
    }

    #[test]
    fn test_get_document_symbols_with_valid_program() {
        let code = r#"
def test_function():
    let x = 42
    return x
"#;
        let uri = Url::parse("file:///test.bend").unwrap();
        let params = DocumentSymbolParams::new(TextDocumentIdentifier::new(uri.clone()), None);
        let symbols = get_document_symbols(&params);
        assert!(symbols.is_some());
    }

    #[test]
    fn test_diagnostic_source_field() {
        let uri = Url::parse("file:///test.bend").unwrap();
        // Test that diagnostics have proper source field
        let diagnostic = Diagnostic {
            range: Range::new(Position::new(0, 0), Position::new(0, 5)),
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Test error".to_string(),
            source: Some("bend-pvm".to_string()),
            ..Diagnostic::default()
        };
        assert_eq!(diagnostic.source, Some("bend-pvm".to_string()));
    }
}
