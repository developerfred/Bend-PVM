// Test script for parser functionality

fn main() {
    println!("Testing parser functionality...");

    let test_source = r#"
fn add(x: u24, y: u24) -> u24 {
    x + y
}
"#;

    println!("Test source code:");
    println!("{}", test_source);

    // Test basic lexer functionality
    use bend_pvm::compiler::lexer::lexer::BendLexer;

    let mut lexer = BendLexer::new(test_source);
    let mut token_count = 0;

    while let Some(token) = lexer.next_token().token {
        if token == bend_pvm::compiler::lexer::token::Token::EOF {
            break;
        }
        token_count += 1;
        println!("Token: {:?}", token);
        if token_count > 20 {
            println!("... (stopping after 20 tokens)");
            break;
        }
    }

    println!("Lexer test completed - found {} tokens", token_count);

    // Test basic parser functionality
    use bend_pvm::compiler::parser::parser::Parser;

    let mut parser = Parser::new(test_source);
    match parser.parse_program() {
        Ok(program) => {
            println!("Parser test PASSED!");
            println!("Parsed {} definitions", program.definitions.len());
            println!("Parsed {} imports", program.imports.len());
        }
        Err(err) => {
            println!("Parser test FAILED: {:?}", err);
        }
    }
}
