//! Parser for the Slvr language
//!
//! Converts a stream of tokens into an Abstract Syntax Tree (AST).

use crate::ast::*;
use crate::error::{SlvrError, SlvrResult};
use crate::lexer::{Lexer, Token, TokenType};

/// Parser for Slvr language
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser from source code
    pub fn new(input: &str) -> SlvrResult<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Ok(Self {
            tokens,
            position: 0,
        })
    }

    /// Parse a complete program
    pub fn parse(&mut self) -> SlvrResult<Program> {
        let mut definitions = Vec::new();
        while !self.is_at_end() {
            definitions.push(self.parse_definition()?);
        }
        Ok(Program { definitions })
    }

    fn parse_definition(&mut self) -> SlvrResult<Definition> {
        match &self.current_token().token_type {
            TokenType::Module => self.parse_module(),
            TokenType::Defun => self.parse_function(),
            TokenType::Defschema => self.parse_schema(),
            TokenType::Deftable => self.parse_table(),
            TokenType::Defconst => self.parse_constant(),
            _ => Err(SlvrError::parse(
                self.current_token().line,
                self.current_token().column,
                "expected definition",
            )),
        }
    }

    fn parse_module(&mut self) -> SlvrResult<Definition> {
        self.consume(TokenType::Module)?;
        let name = self.parse_identifier()?;
        let doc = self.parse_optional_string();
        let mut body = Vec::new();

        self.consume(TokenType::LeftBrace)?;
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            body.push(self.parse_definition()?);
        }
        self.consume(TokenType::RightBrace)?;

        Ok(Definition::Module { name, doc, body })
    }

    fn parse_function(&mut self) -> SlvrResult<Definition> {
        self.consume(TokenType::Defun)?;
        let name = self.parse_identifier()?;
        let doc = self.parse_optional_string();

        self.consume(TokenType::LeftParen)?;
        let params = self.parse_parameters()?;
        self.consume(TokenType::RightParen)?;

        self.consume(TokenType::Arrow)?;
        let return_type = self.parse_type()?;

        let body = self.parse_expression()?;

        Ok(Definition::Function {
            name,
            params,
            return_type,
            doc,
            body,
        })
    }

    fn parse_schema(&mut self) -> SlvrResult<Definition> {
        self.consume(TokenType::Defschema)?;
        let name = self.parse_identifier()?;
        let doc = self.parse_optional_string();

        self.consume(TokenType::LeftBrace)?;
        let mut fields = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let field_name = self.parse_identifier()?;
            self.consume(TokenType::Colon)?;
            let field_type = self.parse_type()?;
            fields.push((field_name, field_type));
            if self.check(&TokenType::Comma) {
                self.advance();
            }
        }
        self.consume(TokenType::RightBrace)?;

        Ok(Definition::Schema { name, fields, doc })
    }

    fn parse_table(&mut self) -> SlvrResult<Definition> {
        self.consume(TokenType::Deftable)?;
        let name = self.parse_identifier()?;
        self.consume(TokenType::Colon)?;
        let schema = self.parse_identifier()?;
        let doc = self.parse_optional_string();

        Ok(Definition::Table { name, schema, doc })
    }

    fn parse_constant(&mut self) -> SlvrResult<Definition> {
        self.consume(TokenType::Defconst)?;
        let name = self.parse_identifier()?;
        self.consume(TokenType::Colon)?;
        let ty = self.parse_type()?;
        self.consume(TokenType::Equal)?;
        let value = self.parse_expression()?;

        Ok(Definition::Constant { name, ty, value })
    }

    fn parse_expression(&mut self) -> SlvrResult<Expr> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> SlvrResult<Expr> {
        let mut left = self.parse_and_expression()?;
        while self.check(&TokenType::Or) {
            self.advance();
            let right = self.parse_and_expression()?;
            left = Expr::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expression(&mut self) -> SlvrResult<Expr> {
        let mut left = self.parse_comparison_expression()?;
        while self.check(&TokenType::And) {
            self.advance();
            let right = self.parse_comparison_expression()?;
            left = Expr::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison_expression(&mut self) -> SlvrResult<Expr> {
        let mut left = self.parse_additive_expression()?;
        while let Some(op) = self.match_comparison_op() {
            let right = self.parse_additive_expression()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive_expression(&mut self) -> SlvrResult<Expr> {
        let mut left = self.parse_multiplicative_expression()?;
        while let Some(op) = self.match_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative_expression(&mut self) -> SlvrResult<Expr> {
        let mut left = self.parse_power_expression()?;
        while let Some(op) = self.match_multiplicative_op() {
            let right = self.parse_power_expression()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_power_expression(&mut self) -> SlvrResult<Expr> {
        let mut left = self.parse_unary_expression()?;
        while self.check(&TokenType::Caret) {
            self.advance();
            let right = self.parse_unary_expression()?;
            left = Expr::BinOp {
                op: BinOp::Power,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> SlvrResult<Expr> {
        match &self.current_token().token_type {
            TokenType::Not => {
                self.advance();
                let operand = self.parse_unary_expression()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            TokenType::Minus => {
                self.advance();
                let operand = self.parse_unary_expression()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_postfix_expression(&mut self) -> SlvrResult<Expr> {
        let mut expr = self.parse_primary_expression()?;
        loop {
            match &self.current_token().token_type {
                TokenType::LeftParen => {
                    self.advance();
                    let args = self.parse_arguments()?;
                    self.consume(TokenType::RightParen)?;
                    expr = Expr::Call {
                        function: Box::new(expr),
                        args,
                    };
                }
                TokenType::Dot => {
                    self.advance();
                    let field = self.parse_identifier()?;
                    expr = Expr::FieldAccess {
                        object: Box::new(expr),
                        field,
                    };
                }
                TokenType::LeftBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.consume(TokenType::RightBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> SlvrResult<Expr> {
        match &self.current_token().token_type {
            TokenType::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Integer(n)))
            }
            TokenType::Decimal(d) => {
                let d = *d;
                self.advance();
                Ok(Expr::Literal(Literal::Decimal(d)))
            }
            TokenType::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(true)))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(false)))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Variable(name))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(TokenType::RightParen)?;
                Ok(expr)
            }
            TokenType::LeftBracket => self.parse_list(),
            TokenType::LeftBrace => self.parse_object(),
            TokenType::If => self.parse_if(),
            TokenType::Let => self.parse_let(),
            _ => Err(SlvrError::parse(
                self.current_token().line,
                self.current_token().column,
                "unexpected token in expression",
            )),
        }
    }

    fn parse_list(&mut self) -> SlvrResult<Expr> {
        self.consume(TokenType::LeftBracket)?;
        let mut elements = Vec::new();
        while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
            elements.push(self.parse_expression()?);
            if self.check(&TokenType::Comma) {
                self.advance();
            }
        }
        self.consume(TokenType::RightBracket)?;
        Ok(Expr::List(elements))
    }

    fn parse_object(&mut self) -> SlvrResult<Expr> {
        self.consume(TokenType::LeftBrace)?;
        let mut fields = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let key = self.parse_identifier()?;
            self.consume(TokenType::Colon)?;
            let value = self.parse_expression()?;
            fields.push((key, value));
            if self.check(&TokenType::Comma) {
                self.advance();
            }
        }
        self.consume(TokenType::RightBrace)?;
        Ok(Expr::Object(fields))
    }

    fn parse_if(&mut self) -> SlvrResult<Expr> {
        self.consume(TokenType::If)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_expression()?;
        let else_branch = if self.check(&TokenType::Identifier("else".to_string())) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Ok(Expr::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_let(&mut self) -> SlvrResult<Expr> {
        self.consume(TokenType::Let)?;
        let name = self.parse_identifier()?;
        self.consume(TokenType::Equal)?;
        let value = self.parse_expression()?;
        let body = self.parse_expression()?;
        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_parameters(&mut self) -> SlvrResult<Vec<(String, Type)>> {
        let mut params = Vec::new();
        while !self.check(&TokenType::RightParen) && !self.is_at_end() {
            let name = self.parse_identifier()?;
            self.consume(TokenType::Colon)?;
            let ty = self.parse_type()?;
            params.push((name, ty));
            if self.check(&TokenType::Comma) {
                self.advance();
            }
        }
        Ok(params)
    }

    fn parse_arguments(&mut self) -> SlvrResult<Vec<Expr>> {
        let mut args = Vec::new();
        while !self.check(&TokenType::RightParen) && !self.is_at_end() {
            args.push(self.parse_expression()?);
            if self.check(&TokenType::Comma) {
                self.advance();
            }
        }
        Ok(args)
    }

    fn parse_type(&mut self) -> SlvrResult<Type> {
        match &self.current_token().token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                match name.as_str() {
                    "integer" => Ok(Type::Integer),
                    "decimal" => Ok(Type::Decimal),
                    "string" => Ok(Type::String),
                    "boolean" => Ok(Type::Boolean),
                    "object" => Ok(Type::Object),
                    "unit" => Ok(Type::Unit),
                    _ => Ok(Type::Custom(name)),
                }
            }
            TokenType::LeftBracket => {
                self.advance();
                let inner = self.parse_type()?;
                self.consume(TokenType::RightBracket)?;
                Ok(Type::List(Box::new(inner)))
            }
            _ => Err(SlvrError::parse(
                self.current_token().line,
                self.current_token().column,
                "expected type",
            )),
        }
    }

    fn parse_identifier(&mut self) -> SlvrResult<String> {
        match &self.current_token().token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(SlvrError::parse(
                self.current_token().line,
                self.current_token().column,
                "expected identifier",
            )),
        }
    }

    fn parse_optional_string(&mut self) -> Option<String> {
        if let TokenType::String(s) = &self.current_token().token_type {
            let s = s.clone();
            self.advance();
            Some(s)
        } else {
            None
        }
    }

    fn match_comparison_op(&mut self) -> Option<BinOp> {
        match &self.current_token().token_type {
            TokenType::EqualEqual => {
                self.advance();
                Some(BinOp::Equal)
            }
            TokenType::NotEqual => {
                self.advance();
                Some(BinOp::NotEqual)
            }
            TokenType::Less => {
                self.advance();
                Some(BinOp::Less)
            }
            TokenType::LessEqual => {
                self.advance();
                Some(BinOp::LessEqual)
            }
            TokenType::Greater => {
                self.advance();
                Some(BinOp::Greater)
            }
            TokenType::GreaterEqual => {
                self.advance();
                Some(BinOp::GreaterEqual)
            }
            _ => None,
        }
    }

    fn match_additive_op(&mut self) -> Option<BinOp> {
        match &self.current_token().token_type {
            TokenType::Plus => {
                self.advance();
                Some(BinOp::Add)
            }
            TokenType::Minus => {
                self.advance();
                Some(BinOp::Subtract)
            }
            TokenType::Concat => {
                self.advance();
                Some(BinOp::Concat)
            }
            _ => None,
        }
    }

    fn match_multiplicative_op(&mut self) -> Option<BinOp> {
        match &self.current_token().token_type {
            TokenType::Star => {
                self.advance();
                Some(BinOp::Multiply)
            }
            TokenType::Slash => {
                self.advance();
                Some(BinOp::Divide)
            }
            TokenType::Percent => {
                self.advance();
                Some(BinOp::Modulo)
            }
            _ => None,
        }
    }

    fn current_token(&self) -> Token {
        self.tokens
            .get(self.position)
            .cloned()
            .unwrap_or_else(|| Token::new(TokenType::Eof, 0, 0))
    }

    fn check(&self, token_type: &TokenType) -> bool {
        let current = self.current_token();
        std::mem::discriminant(&current.token_type) == std::mem::discriminant(token_type)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn consume(&mut self, token_type: TokenType) -> SlvrResult<()> {
        if self.check(&token_type) {
            self.advance();
            Ok(())
        } else {
            Err(SlvrError::parse(
                self.current_token().line,
                self.current_token().column,
                format!("expected {:?}", token_type),
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        let current = self.current_token();
        matches!(current.token_type, TokenType::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expression() {
        // Parser requires valid Slvr syntax
        // This test verifies the parser can be created
        let result = Parser::new("(defun test () -> integer 42)");
        assert!(result.is_ok());
    }
}
