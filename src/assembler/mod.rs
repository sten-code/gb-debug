mod lexer;
pub use lexer::*;
mod parser;
pub use parser::*;

pub fn assemble(asm: &str) -> Vec<FullInstruction> {
    let mut lexer = Lexer::new(asm);
    lexer.tokenize();

    let mut parser = Parser::new(lexer.tokens);
    parser.parse();

    parser.instructions
}
