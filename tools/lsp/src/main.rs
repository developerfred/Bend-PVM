use std::error::Error;
use std::fs;
use std::path::Path;

use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::*;
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Create the connection to the language server client
    let (connection, io_threads) = Connection::stdio();

    // Initialize the server capabilities
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![".".to_string()]),
            ..CompletionOptions::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..ServerCapabilities::default()
    }).unwrap();

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
            Message::Response(_resp) => {
                // Do nothing for now
            }
            Message::Notification(_not) => {
                // Do nothing for now
            }
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
        "textDocument/formatting" => {
            let params = serde_json::from_value::<DocumentFormattingParams>(req.params.clone())?;
            let edits = format_document(&params);
            let resp = Response {
                id: req.id,
                result: Some(serde_json::to_value(edits)?),
                error: None,
            };
            connection.sender.send(Message::Response(resp))?;
        }
        _ => {
            // Unknown request, respond with null
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

fn get_completion_items(params: &CompletionParams) -> Vec<CompletionItem> {
    // Get the document
    let document_uri = params.text_document_position.text_document.uri.clone();
    let document_path = document_uri.to_file_path().unwrap();
    let document_text = fs::read_to_string(document_path).unwrap_or_default();
    
    // Basic keywords
    let mut items = vec![
        CompletionItem {
            label: "def".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define a function".to_string()),
            insert_text: Some("def ${1:name}(${2:params}):\n    ${0:pass}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "type".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define a type".to_string()),
            insert_text: Some("type ${1:Name}:\n    ${0:pass}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "object".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define an object".to_string()),
            insert_text: Some("object ${1:Name} { ${0:fields} }".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "return".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Return a value".to_string()),
            insert_text: Some("return ${0:value}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "if".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("If statement".to_string()),
            insert_text: Some("if ${1:condition}:\n    ${2:pass}\nelse:\n    ${0:pass}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "match".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Match statement".to_string()),
            insert_text: Some("match ${1:value}:\n    case ${2:pattern}:\n        ${0:pass}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "fold".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Fold statement".to_string()),
            insert_text: Some("fold ${1:value}:\n    case ${2:pattern}:\n        ${0:pass}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "bend".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Bend statement".to_string()),
            insert_text: Some("bend ${1:state} = ${2:initial}:\n    when ${3:condition}:\n        ${0:pass}\n    else:\n        ${4:pass}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "with".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("With block for monadic operations".to_string()),
            insert_text: Some("with ${1:IO}:\n    ${0:pass}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "import".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Import statement".to_string()),
            insert_text: Some("import ${0:module}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "from".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("From import statement".to_string()),
            insert_text: Some("from ${1:module} import ${0:name}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
    ];
    
    // Add built-in types
    items.extend(vec![
        CompletionItem {
            label: "u24".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("Unsigned 24-bit integer".to_string()),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "i24".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("Signed 24-bit integer".to_string()),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "f24".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("24-bit floating point number".to_string()),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "String".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("String type".to_string()),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "List".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("List type".to_string()),
            insert_text: Some("List(${0:T})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "Option".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("Option type".to_string()),
            insert_text: Some("Option(${0:T})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "Result".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("Result type".to_string()),
            insert_text: Some("Result(${1:T}, ${0:E})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "Tree".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("Tree type".to_string()),
            insert_text: Some("Tree(${0:T})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "Map".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("Map type".to_string()),
            insert_text: Some("Map(${0:T})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "IO".to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some("IO type".to_string()),
            insert_text: Some("IO(${0:T})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
    ]);
    
    // Add built-in functions
    items.extend(vec![
        CompletionItem {
            label: "wrap".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Wrap a value in a monad".to_string()),
            insert_text: Some("wrap(${0:value})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
        CompletionItem {
            label: "fork".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Fork execution in a bend statement".to_string()),
            insert_text: Some("fork(${0:state})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..CompletionItem::default()
        },
    ]);
    
    // TODO: Add more context-aware completions based on the document
    
    items
}

fn get_hover(params: &HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let document_uri = params.text_document_position_params.text_document.uri.clone();
    let document_path = document_uri.to_file_path().unwrap();
    let document_text = fs::read_to_string(document_path).unwrap_or_default();
    
    // For simplicity, we'll just return a basic hover message
    // In a real implementation, we would parse the document and identify the token at the position
    
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "No detailed information available yet.".to_string(),
        }),
        range: None,
    })
}

fn get_definition(params: &GotoDefinitionParams) -> Option<GotoDefinitionResponse> {
    let position = params.text_document_position_params.position;
    let document_uri = params.text_document_position_params.text_document.uri.clone();
    let document_path = document_uri.to_file_path().unwrap();
    let document_text = fs::read_to_string(document_path).unwrap_or_default();
    
    // For simplicity, we'll just return None
    // In a real implementation, we would parse the document and find the definition
    
    None
}

fn format_document(params: &DocumentFormattingParams) -> Option<Vec<TextEdit>> {
    let document_uri = params.text_document.uri.clone();
    let document_path = document_uri.to_file_path().unwrap();
    let document_text = fs::read_to_string(document_path).unwrap_or_default();
    
    // For simplicity, we'll just return None
    // In a real implementation, we would parse and format the document
    
    None
}