use crate::ast;
use crate::{
    OptionSpanned as _, Parse, ParseError, ParseErrorKind, Parser, Peek, Peeker, Spanned, ToTokens,
};
use std::mem::take;
use std::ops;

/// Indicator that an expression should be parsed with an eager brace.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EagerBrace(pub(crate) bool);

impl ops::Deref for EagerBrace {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Indicator that an expression should be parsed as an eager binary expression.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EagerBinary(pub(crate) bool);

impl ops::Deref for EagerBinary {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A rune expression.
#[derive(Debug, Clone, ToTokens, Spanned, PartialEq, Eq)]
pub enum Expr {
    /// An path expression.
    Path(ast::Path),
    /// A declaration.
    Item(ast::Item),
    /// An assign expression.
    ExprAssign(ast::ExprAssign),
    /// A while loop.
    ExprWhile(ast::ExprWhile),
    /// An unconditional loop.
    ExprLoop(ast::ExprLoop),
    /// An for loop.
    ExprFor(ast::ExprFor),
    /// A let expression.
    ExprLet(ast::ExprLet),
    /// An if expression.
    ExprIf(ast::ExprIf),
    /// An match expression.
    ExprMatch(ast::ExprMatch),
    /// A function call,
    ExprCall(ast::ExprCall),
    /// A macro call,
    MacroCall(ast::MacroCall),
    /// A field access on an expression.
    ExprFieldAccess(ast::ExprFieldAccess),
    /// A grouped expression.
    ExprGroup(ast::ExprGroup),
    /// A binary expression.
    ExprBinary(ast::ExprBinary),
    /// A unary expression.
    ExprUnary(ast::ExprUnary),
    /// An index set operation.
    ExprIndex(ast::ExprIndex),
    /// A break expression.
    ExprBreak(ast::ExprBreak),
    /// A yield expression.
    ExprYield(ast::ExprYield),
    /// A block as an expression.
    ExprBlock(ast::ExprBlock),
    /// A return statement.
    ExprReturn(ast::ExprReturn),
    /// An await expression.
    ExprAwait(ast::ExprAwait),
    /// Try expression.
    ExprTry(ast::ExprTry),
    /// A select expression.
    ExprSelect(ast::ExprSelect),
    /// A closure expression.
    ExprClosure(ast::ExprClosure),
    /// A literal expression.
    ExprLit(ast::ExprLit),
}

impl Expr {
    /// Indicates if an expression needs a semicolon or must be last in a block.
    pub fn needs_semi(&self) -> bool {
        match self {
            Self::ExprWhile(_) => false,
            Self::ExprLoop(_) => false,
            Self::ExprFor(_) => false,
            Self::ExprIf(_) => false,
            Self::ExprMatch(_) => false,
            Self::ExprBlock(_) => false,
            Self::ExprSelect(_) => false,
            _ => true,
        }
    }

    /// Test if expression should be chained by default.
    pub fn is_chainable(&self) -> bool {
        match self {
            Self::ExprWhile(..) => false,
            Self::ExprLoop(..) => false,
            Self::ExprFor(..) => false,
            _ => true,
        }
    }

    /// Take the attributes from the expression.
    pub fn take_attributes(&mut self) -> Vec<ast::Attribute> {
        match self {
            Expr::Path(_) => Vec::new(),
            Expr::Item(item) => item.take_attributes(),
            Expr::ExprBreak(expr) => take(&mut expr.attributes),
            Expr::ExprYield(expr) => take(&mut expr.attributes),
            Expr::ExprBlock(expr) => take(&mut expr.attributes),
            Expr::ExprReturn(expr) => take(&mut expr.attributes),
            Expr::ExprClosure(expr) => take(&mut expr.attributes),
            Expr::ExprMatch(expr) => take(&mut expr.attributes),
            Expr::ExprWhile(expr) => take(&mut expr.attributes),
            Expr::ExprLoop(expr) => take(&mut expr.attributes),
            Expr::ExprFor(expr) => take(&mut expr.attributes),
            Expr::ExprLet(expr) => take(&mut expr.attributes),
            Expr::ExprIf(expr) => take(&mut expr.attributes),
            Expr::ExprSelect(expr) => take(&mut expr.attributes),
            Expr::ExprLit(expr) => take(&mut expr.attributes),
            Expr::ExprAssign(expr) => take(&mut expr.attributes),
            Expr::ExprBinary(expr) => take(&mut expr.attributes),
            Expr::ExprCall(expr) => take(&mut expr.attributes),
            Expr::MacroCall(expr) => take(&mut expr.attributes),
            Expr::ExprFieldAccess(expr) => take(&mut expr.attributes),
            Expr::ExprGroup(expr) => take(&mut expr.attributes),
            Expr::ExprUnary(expr) => take(&mut expr.attributes),
            Expr::ExprIndex(expr) => take(&mut expr.attributes),
            Expr::ExprAwait(expr) => take(&mut expr.attributes),
            Expr::ExprTry(expr) => take(&mut expr.attributes),
        }
    }

    /// Access the attributes of the expression.
    pub fn attributes(&self) -> &[ast::Attribute] {
        match self {
            Expr::Path(_) => &[],
            Expr::Item(expr) => expr.attributes(),
            Expr::ExprBreak(expr) => &expr.attributes,
            Expr::ExprYield(expr) => &expr.attributes,
            Expr::ExprBlock(expr) => &expr.attributes,
            Expr::ExprReturn(expr) => &expr.attributes,
            Expr::ExprClosure(expr) => &expr.attributes,
            Expr::ExprMatch(expr) => &expr.attributes,
            Expr::ExprWhile(expr) => &expr.attributes,
            Expr::ExprLoop(expr) => &expr.attributes,
            Expr::ExprFor(expr) => &expr.attributes,
            Expr::ExprLet(expr) => &expr.attributes,
            Expr::ExprIf(expr) => &expr.attributes,
            Expr::ExprSelect(expr) => &expr.attributes,
            Expr::ExprLit(expr) => &expr.attributes,
            Expr::ExprAssign(expr) => &expr.attributes,
            Expr::ExprBinary(expr) => &expr.attributes,
            Expr::ExprCall(expr) => &expr.attributes,
            Expr::MacroCall(expr) => &expr.attributes,
            Expr::ExprFieldAccess(expr) => &expr.attributes,
            Expr::ExprGroup(expr) => &expr.attributes,
            Expr::ExprUnary(expr) => &expr.attributes,
            Expr::ExprIndex(expr) => &expr.attributes,
            Expr::ExprAwait(expr) => &expr.attributes,
            Expr::ExprTry(expr) => &expr.attributes,
        }
    }

    /// Parse an expression without an eager brace.
    ///
    /// This is used to solve a syntax ambiguity when parsing expressions that
    /// are arguments to statements immediately followed by blocks. Like `if`,
    /// `while`, and `match`.
    pub(crate) fn parse_without_eager_brace(p: &mut Parser<'_>) -> Result<Self, ParseError> {
        Self::parse_with(p, EagerBrace(false), EagerBinary(true))
    }

    /// ull, configurable parsing of an expression.F
    pub(crate) fn parse_with(
        p: &mut Parser<'_>,
        eager_brace: EagerBrace,
        eager_binary: EagerBinary,
    ) -> Result<Self, ParseError> {
        let mut attributes = p.parse()?;

        let expr = Self::parse_base(p, &mut attributes, eager_brace)?;
        let expr = Self::parse_chain(p, expr)?;

        let expr = if *eager_binary {
            Self::parse_binary(p, expr, 0, eager_brace)?
        } else {
            expr
        };

        if let Some(span) = attributes.option_span() {
            return Err(ParseError::new(
                span,
                ParseErrorKind::AttributesNotSupported,
            ));
        }

        Ok(expr)
    }

    /// Parse expressions that start with an identifier.
    pub(crate) fn parse_with_meta_path(
        p: &mut Parser<'_>,
        attributes: &mut Vec<ast::Attribute>,
        path: ast::Path,
        eager_brace: EagerBrace,
    ) -> Result<Self, ParseError> {
        if *eager_brace && p.peek::<T!['{']>()? {
            let ident = ast::LitObjectIdent::Named(path);

            return Ok(Self::ExprLit(ast::ExprLit {
                attributes: take(attributes),
                lit: ast::Lit::Object(ast::LitObject::parse_with_ident(p, ident)?),
            }));
        }

        if p.peek::<T![!]>()? {
            return Ok(Self::MacroCall(ast::MacroCall::parse_with_meta_path(
                p,
                std::mem::take(attributes),
                path,
            )?));
        }

        Ok(Self::Path(path))
    }

    /// Parsing something that opens with a parenthesis.
    pub fn parse_open_paren(
        p: &mut Parser<'_>,
        attributes: &mut Vec<ast::Attribute>,
    ) -> Result<Self, ParseError> {
        if p.peek::<T![()]>()? {
            return Ok(Self::ExprLit(ast::ExprLit {
                attributes: take(attributes),
                lit: p.parse()?,
            }));
        }

        let open = p.parse::<ast::OpenParen>()?;
        let expr = ast::Expr::parse_with(p, EagerBrace(true), EagerBinary(true))?;

        if p.peek::<T![')']>()? {
            return Ok(Expr::ExprGroup(ast::ExprGroup {
                attributes: take(attributes),
                open,
                expr: Box::new(expr),
                close: p.parse()?,
            }));
        }

        let tuple = ast::LitTuple::parse_from_first_expr(p, open, expr)?;

        Ok(Expr::ExprLit(ast::ExprLit {
            attributes: take(attributes),
            lit: ast::Lit::Tuple(tuple),
        }))
    }

    pub(crate) fn parse_with_meta(
        p: &mut Parser<'_>,
        attributes: &mut Vec<ast::Attribute>,
        path: Option<ast::Path>,
    ) -> Result<Self, ParseError> {
        let lhs = if let Some(path) = path {
            Self::parse_with_meta_path(p, attributes, path, EagerBrace(true))?
        } else {
            Self::parse_base(p, attributes, EagerBrace(true))?
        };

        let lhs = Self::parse_chain(p, lhs)?;
        Ok(Self::parse_binary(p, lhs, 0, EagerBrace(true))?)
    }

    /// Parse a basic expression.
    fn parse_base(
        p: &mut Parser<'_>,
        attributes: &mut Vec<ast::Attribute>,
        eager_brace: EagerBrace,
    ) -> Result<Self, ParseError> {
        if let Some(path) = p.parse::<Option<ast::Path>>()? {
            return Ok(Self::parse_with_meta_path(
                p,
                attributes,
                path,
                eager_brace,
            )?);
        }

        if ast::Lit::peek_in_expr(p.peeker()) {
            return Ok(ast::Expr::ExprLit(ast::ExprLit::parse_with_meta(
                p,
                take(attributes),
            )?));
        }

        let mut label = p.parse::<Option<(ast::Label, T![:])>>()?;
        let mut async_token = p.parse::<Option<T![async]>>()?;

        let expr = match p.nth(0)? {
            K![||] | K![|] => Self::ExprClosure(ast::ExprClosure::parse_with_attributes_and_async(
                p,
                take(attributes),
                take(&mut async_token),
            )?),
            K![select] => {
                Self::ExprSelect(ast::ExprSelect::parse_with_attributes(p, take(attributes))?)
            }
            K![!] | K![-] | K![&] | K![*] => Self::ExprUnary(ast::ExprUnary::parse_with_meta(
                p,
                take(attributes),
                eager_brace,
            )?),
            K![while] => Self::ExprWhile(ast::ExprWhile::parse_with_meta(
                p,
                take(attributes),
                take(&mut label),
            )?),
            K![loop] => Self::ExprLoop(ast::ExprLoop::parse_with_meta(
                p,
                take(attributes),
                take(&mut label),
            )?),
            K![for] => Self::ExprFor(ast::ExprFor::parse_with_meta(
                p,
                take(attributes),
                take(&mut label),
            )?),
            K![let] => Self::ExprLet(ast::ExprLet::parse_with_meta(p, take(attributes))?),
            K![if] => Self::ExprIf(ast::ExprIf::parse_with_meta(p, take(attributes))?),
            K![match] => {
                Self::ExprMatch(ast::ExprMatch::parse_with_attributes(p, take(attributes))?)
            }
            K!['('] => Self::parse_open_paren(p, attributes)?,
            K!['{'] => Self::ExprBlock(ast::ExprBlock {
                async_token: take(&mut async_token),
                attributes: take(attributes),
                block: p.parse()?,
            }),
            K![break] => Self::ExprBreak(ast::ExprBreak::parse_with_meta(p, take(attributes))?),
            K![yield] => Self::ExprYield(ast::ExprYield::parse_with_meta(p, take(attributes))?),
            K![return] => Self::ExprReturn(ast::ExprReturn::parse_with_meta(p, take(attributes))?),
            _ => {
                return Err(ParseError::expected(p.token(0)?, "expression"));
            }
        };

        if let Some(span) = label.option_span() {
            return Err(ParseError::new(span, ParseErrorKind::UnsupportedLabel));
        }

        if let Some(span) = async_token.option_span() {
            return Err(ParseError::new(span, ParseErrorKind::UnsupportedAsync));
        }

        Ok(expr)
    }

    /// Parse an expression chain.
    fn parse_chain(p: &mut Parser<'_>, mut expr: Self) -> Result<Self, ParseError> {
        while !p.is_eof()? {
            let is_chainable = expr.is_chainable();

            match p.nth(0)? {
                K!['['] if is_chainable => {
                    expr = Self::ExprIndex(ast::ExprIndex {
                        attributes: expr.take_attributes(),
                        target: Box::new(expr),
                        open: p.parse()?,
                        index: p.parse()?,
                        close: p.parse()?,
                    });
                }
                // Chained function call.
                K!['('] if is_chainable => {
                    let args = p.parse::<ast::Parenthesized<ast::Expr, T![,]>>()?;

                    expr = Expr::ExprCall(ast::ExprCall {
                        id: Default::default(),
                        attributes: expr.take_attributes(),
                        expr: Box::new(expr),
                        args,
                    });
                }
                K![?] => {
                    expr = Expr::ExprTry(ast::ExprTry {
                        attributes: expr.take_attributes(),
                        expr: Box::new(expr),
                        try_token: p.parse()?,
                    });
                }
                K![=] => {
                    let eq = p.parse()?;
                    let rhs = Self::parse_with(p, EagerBrace(true), EagerBinary(true))?;

                    expr = Expr::ExprAssign(ast::ExprAssign {
                        attributes: expr.take_attributes(),
                        lhs: Box::new(expr),
                        eq,
                        rhs: Box::new(rhs),
                    });
                }
                K![.] => {
                    let dot = p.parse()?;

                    if matches!(p.nth(0)?, K![await]) {
                        expr = Expr::ExprAwait(ast::ExprAwait {
                            attributes: expr.take_attributes(),
                            expr: Box::new(expr),
                            dot,
                            await_token: p.parse()?,
                        });

                        continue;
                    }

                    let next = Expr::parse_base(p, &mut vec![], EagerBrace(false))?;

                    let span = match next {
                        Expr::Path(path) => {
                            let span = path.span();

                            if let Some(name) = path.try_as_ident() {
                                expr = Expr::ExprFieldAccess(ast::ExprFieldAccess {
                                    attributes: expr.take_attributes(),
                                    expr: Box::new(expr),
                                    dot,
                                    expr_field: ast::ExprField::Ident(*name),
                                });

                                continue;
                            }

                            span
                        }
                        Expr::ExprLit(ast::ExprLit {
                            lit: ast::Lit::Number(n),
                            attributes,
                        }) if attributes.is_empty() => {
                            expr = Expr::ExprFieldAccess(ast::ExprFieldAccess {
                                attributes: expr.take_attributes(),
                                expr: Box::new(expr),
                                dot,
                                expr_field: ast::ExprField::LitNumber(n),
                            });

                            continue;
                        }
                        other => other.span(),
                    };

                    return Err(ParseError::new(
                        span,
                        ParseErrorKind::UnsupportedFieldAccess,
                    ));
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse a binary expression.
    fn parse_binary(
        p: &mut Parser<'_>,
        mut lhs: Self,
        min_precedence: usize,
        eager_brace: EagerBrace,
    ) -> Result<Self, ParseError> {
        let mut lookahead_tok = ast::BinOp::from_peeker(p.peeker());

        loop {
            let op = match lookahead_tok {
                Some(op) if op.precedence() >= min_precedence => op,
                _ => break,
            };

            let (t1, t2) = op.advance(p)?;

            let rhs = Self::parse_base(p, &mut vec![], eager_brace)?;
            let mut rhs = Self::parse_chain(p, rhs)?;

            lookahead_tok = ast::BinOp::from_peeker(p.peeker());

            loop {
                let lh = match lookahead_tok {
                    Some(lh) if lh.precedence() > op.precedence() => lh,
                    Some(lh) if lh.precedence() == op.precedence() && !op.is_assoc() => {
                        return Err(ParseError::new(
                            lhs.span().join(rhs.span()),
                            ParseErrorKind::PrecedenceGroupRequired,
                        ));
                    }
                    _ => break,
                };

                rhs = Self::parse_binary(p, rhs, lh.precedence(), eager_brace)?;
                lookahead_tok = ast::BinOp::from_peeker(p.peeker());
            }

            lhs = Expr::ExprBinary(ast::ExprBinary {
                attributes: Vec::new(),
                lhs: Box::new(lhs),
                t1,
                t2,
                rhs: Box::new(rhs),
                op,
            });
        }

        Ok(lhs)
    }
}

/// Parsing a block expression.
///
/// # Examples
///
/// ```rust
/// use rune::{testing, ast};
///
/// testing::roundtrip::<ast::Expr>("foo[\"foo\"]");
/// testing::roundtrip::<ast::Expr>("foo.bar()");
/// testing::roundtrip::<ast::Expr>("var()");
/// testing::roundtrip::<ast::Expr>("var");
/// testing::roundtrip::<ast::Expr>("42");
/// testing::roundtrip::<ast::Expr>("1 + 2 / 3 - 4 * 1");
/// testing::roundtrip::<ast::Expr>("foo[\"bar\"]");
/// testing::roundtrip::<ast::Expr>("let var = 42");
/// testing::roundtrip::<ast::Expr>("let var = \"foo bar\"");
/// testing::roundtrip::<ast::Expr>("var[\"foo\"] = \"bar\"");
/// testing::roundtrip::<ast::Expr>("let var = objects[\"foo\"] + 1");
/// testing::roundtrip::<ast::Expr>("var = 42");
///
/// let expr = testing::roundtrip::<ast::Expr>(r#"
///     if 1 { } else { if 2 { } else { } }
/// "#);
/// assert!(matches!(expr, ast::Expr::ExprIf(..)));
///
/// // Chained function calls.
/// testing::roundtrip::<ast::Expr>("foo.bar.baz()");
/// testing::roundtrip::<ast::Expr>("foo[0][1][2]");
/// testing::roundtrip::<ast::Expr>("foo.bar()[0].baz()[1]");
///
/// testing::roundtrip::<ast::Expr>("42 is int::int");
/// testing::roundtrip::<ast::Expr>("{ let x = 1; x }");
///
/// let expr = testing::roundtrip::<ast::Expr>("#[cfg(debug_assertions)] { assert_eq(x, 32); }");
/// assert!(matches!(expr, ast::Expr::ExprBlock(b) if b.attributes.len() == 1 && b.block.statements.len() == 1));
/// ```
impl Parse for Expr {
    fn parse(p: &mut Parser<'_>) -> Result<Self, ParseError> {
        Self::parse_with(p, EagerBrace(true), EagerBinary(true))
    }
}

impl Peek for Expr {
    fn peek(p: &mut Peeker<'_>) -> bool {
        match p.nth(0) {
            K![async] => true,
            K![self] => true,
            K![select] => true,
            ast::Kind::Label(..) => matches!(p.nth(1), K![:]),
            K![#] => true,
            K![-] => true,
            K![!] => true,
            K![&] => true,
            K![*] => true,
            K![while] => true,
            K![loop] => true,
            K![for] => true,
            K![let] => true,
            K![if] => true,
            K![break] => true,
            K![return] => true,
            K![true] => true,
            K![false] => true,
            K![ident(..)] => true,
            K!['('] => true,
            K!['['] => true,
            K!['{'] => true,
            ast::Kind::LitNumber { .. } => true,
            ast::Kind::LitChar { .. } => true,
            ast::Kind::LitByte { .. } => true,
            ast::Kind::LitStr { .. } => true,
            ast::Kind::LitByteStr { .. } => true,
            ast::Kind::Template { .. } => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast, testing};

    #[test]
    fn test_expr_if() {
        testing::roundtrip::<ast::Expr>(r#"if true {} else {}"#);
    }

    #[test]
    fn test_expr_while() {
        testing::roundtrip::<ast::ExprWhile>(r#"while true {}"#);
    }
}
