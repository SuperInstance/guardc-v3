use crate::ast::*;
use crate::error::{CompileError, Result, Span};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Guard,
    Batch,
    Rule,
    Preset,
    Handle,
    With,
    Priority,
    When,
    For,
    Per,
    Second,
    Minute,
    In,
    And,
    Or,
    To,
    Clamped,
    Severity,
    Log,
    Broadcast,
    Shutdown,
    Alert,
    High,
    Medium,
    Low,
    Critical,
    Colon,
    Comma,
    LBrack,
    RBrack,
    LParen,
    RParen,
    Arrow,
    Percent,
    Lte,
    Gte,
    Lt,
    Gt,
    Eq,
    Number(f64),
    Ident(String),
    Unit(Unit),
    Eof,
}

struct Lexer {
    tokens: Vec<(Token, Span)>,
    pos: usize,
}

impl Lexer {
    fn new(input: &str) -> Result<Self> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let mut line = 1;
        let mut col = 1;

        let chars: Vec<char> = input.chars().collect();

        while pos < chars.len() {
            if chars[pos].is_whitespace() {
                if chars[pos] == '\n' { line += 1; col = 1; } else { col += 1; }
                pos += 1;
                continue;
            }
            if chars[pos] == '#' {
                while pos < chars.len() && chars[pos] != '\n' { pos += 1; }
                continue;
            }

            let start = pos;
            let start_col = col;

            match chars[pos] {
                ':' => { tokens.push((Token::Colon, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                ',' => { tokens.push((Token::Comma, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                '[' => { tokens.push((Token::LBrack, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                ']' => { tokens.push((Token::RBrack, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                '(' => { tokens.push((Token::LParen, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                ')' => { tokens.push((Token::RParen, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                '%' => { tokens.push((Token::Percent, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                '=' => { tokens.push((Token::Eq, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                '>' if pos + 1 < chars.len() && chars[pos+1] == '=' => {
                    tokens.push((Token::Gte, Span::new(start, pos+2, line, start_col))); pos += 2; col += 2; continue;
                }
                '<' if pos + 1 < chars.len() && chars[pos+1] == '=' => {
                    tokens.push((Token::Lte, Span::new(start, pos+2, line, start_col))); pos += 2; col += 2; continue;
                }
                '>' => { tokens.push((Token::Gt, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                '<' => { tokens.push((Token::Lt, Span::new(start, pos+1, line, start_col))); pos += 1; col += 1; continue; }
                _ => {}
            }

            // numbers
            if chars[pos].is_ascii_digit() || (chars[pos] == '-' && pos + 1 < chars.len() && chars[pos+1].is_ascii_digit()) {
                let mut num_str = String::new();
                if chars[pos] == '-' { num_str.push('-'); pos += 1; col += 1; }
                while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                    num_str.push(chars[pos]); pos += 1; col += 1;
                }
                let val: f64 = num_str.parse().map_err(|_| CompileError::lex("invalid number", Span::new(start, pos, line, start_col)))?;
                tokens.push((Token::Number(val), Span::new(start, pos, line, start_col)));
                continue;
            }

            // identifiers & keywords
            if chars[pos].is_alphabetic() || chars[pos] == '_' {
                let mut word = String::new();
                while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                    word.push(chars[pos]); pos += 1; col += 1;
                }
                let tok = match word.as_str() {
                    "GUARD" => Token::Guard,
                    "BATCH" => Token::Batch,
                    "RULE" => Token::Rule,
                    "PRESET" => Token::Preset,
                    "HANDLE" => Token::Handle,
                    "with" | "WITH" => Token::With,
                    "priority" | "PRIORITY" => Token::Priority,
                    "when" | "WHEN" => Token::When,
                    "for" | "FOR" => Token::For,
                    "per" | "PER" => Token::Per,
                    "second" | "SECOND" | "seconds" | "SECONDS" => Token::Second,
                    "minute" | "MINUTE" => Token::Minute,
                    "in" | "IN" => Token::In,
                    "AND" => Token::And,
                    "OR" => Token::Or,
                    "TO" | "to" => Token::To,
                    "CLAMPED" => Token::Clamped,
                    "severity" | "SEVERITY" => Token::Severity,
                    "log" | "LOG" => Token::Log,
                    "broadcast" | "BROADCAST" => Token::Broadcast,
                    "shutdown" | "SHUTDOWN" => Token::Shutdown,
                    "alert" | "ALERT" => Token::Alert,
                    "HIGH" => Token::High,
                    "MEDIUM" => Token::Medium,
                    "LOW" => Token::Low,
                    "CRITICAL" => Token::Critical,
                    "PASS" | "Pass" => Token::Ident("PASS".into()),
                    "CAUTION" | "Caution" => Token::Ident("CAUTION".into()),
                    "WARNING" | "Warning" => Token::Ident("WARNING".into()),
                    "kPa" => Token::Unit(Unit::KPa),
                    "MPa" => Token::Unit(Unit::MPa),
                    "PSI" => Token::Unit(Unit::PSI),
                    "value" => Token::Ident("value".into()),
                    "sensor" | "SENSOR" => Token::Ident("SENSOR".into()),
                    "violations" | "VIOLATIONS" => Token::Ident("violations".into()),
                    _ => Token::Ident(word.clone()),
                };
                tokens.push((tok, Span::new(start, pos, line, start_col)));
                continue;
            }

            return Err(CompileError::lex(format!("unexpected character '{}'", chars[pos]), Span::new(start, pos+1, line, start_col)));
        }

        tokens.push((Token::Eof, Span::new(pos, pos, line, col)));
        Ok(Self { tokens, pos: 0 })
    }

    fn peek(&self) -> (Token, Span) {
        self.tokens.get(self.pos).cloned().unwrap_or((Token::Eof, Span::zero()))
    }

    fn advance(&mut self) -> (Token, Span) {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or((Token::Eof, Span::zero()));
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<Span> {
        let (tok, span) = self.advance();
        if std::mem::discriminant(&tok) == std::mem::discriminant(expected) {
            Ok(span)
        } else {
            Err(CompileError::parse(format!("expected {:?}, got {:?}", expected, tok), span))
        }
    }

    fn matches(&self, expected: &Token) -> bool {
        let (tok, _) = self.peek();
        std::mem::discriminant(&tok) == std::mem::discriminant(expected)
    }

    fn matches_ident(&self, s: &str) -> bool {
        let (tok, _) = self.peek();
        matches!(tok, Token::Ident(i) if i == s)
    }
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self> {
        Ok(Self { lexer: Lexer::new(input)? })
    }

    pub fn parse(mut self) -> Result<Program> {
        let mut program = Program {
            guards: Vec::new(),
            rules: Vec::new(),
            presets: Vec::new(),
            handlers: Vec::new(),
        };

        while !self.lexer.matches(&Token::Eof) {
            if self.lexer.matches(&Token::Guard) {
                program.guards.push(self.parse_guard()?);
            } else if self.lexer.matches(&Token::Batch) {
                program.guards.push(self.parse_batch()?);
            } else if self.lexer.matches(&Token::Rule) {
                program.rules.push(self.parse_rule()?);
            } else if self.lexer.matches(&Token::Preset) {
                program.presets.push(self.parse_preset()?);
            } else if self.lexer.matches(&Token::Handle) {
                program.handlers.push(self.parse_handler()?);
            } else {
                let (_, span) = self.lexer.advance();
                return Err(CompileError::parse("expected GUARD, BATCH, RULE, PRESET, or HANDLE", span));
            }
        }

        Ok(program)
    }

    fn parse_guard(&mut self) -> Result<GuardDecl> {
        let _ = self.lexer.expect(&Token::Guard)?;
        let (name_tok, span) = self.lexer.advance();
        let name = match name_tok {
            Token::Ident(s) => s,
            _ => return Err(CompileError::parse("expected constraint name", span)),
        };

        let kind = self.parse_constraint_kind()?;
        let priority = Priority::Medium;
        let mut unit = Unit::None;
        let mut temporal = Temporal::None;
        let mut condition = None;
        let mut final_priority = priority;

        loop {
            if let Token::Unit(ref u) = self.lexer.peek().0 {
                unit = u.clone();
                self.lexer.advance();
                continue;
            }
            if self.lexer.matches(&Token::With) {
                self.lexer.advance();
                if self.lexer.matches(&Token::Priority) {
                    self.lexer.advance();
                    final_priority = self.parse_priority()?;
                }
                continue;
            }
            if self.lexer.matches(&Token::When) {
                self.lexer.advance();
                condition = Some(self.parse_expr()?);
                continue;
            }
            if self.lexer.matches(&Token::Per) {
                self.lexer.advance();
                let _ = self.lexer.advance(); // "second" or "minute"
                temporal = Temporal::PerSecond;
                if self.lexer.matches(&Token::For) {
                    self.lexer.advance();
                    let (tok, _) = self.lexer.advance();
                    let dur = match tok { Token::Number(n) => n as u64, _ => 1 };
                    temporal = Temporal::ForSeconds(dur);
                    let _ = self.lexer.advance(); // "seconds"
                }
                continue;
            }
            if self.lexer.matches(&Token::For) {
                self.lexer.advance();
                let (tok, _) = self.lexer.advance();
                let dur = match tok { Token::Number(n) => n as u64, _ => 1 };
                let _ = self.lexer.advance(); // unit
                temporal = Temporal::ForSeconds(dur);
                continue;
            }
            break;
        }

        Ok(GuardDecl { name, kind, priority: final_priority, unit, temporal, condition, batch_size: None, span })
    }

    fn parse_batch(&mut self) -> Result<GuardDecl> {
        let _ = self.lexer.expect(&Token::Batch)?;
        let (name_tok, span) = self.lexer.advance();
        let name = match name_tok {
            Token::Ident(s) => s,
            _ => return Err(CompileError::parse("expected batch name", span)),
        };

        let batch_size = if self.lexer.matches(&Token::LBrack) {
            self.lexer.advance();
            let (tok, _) = self.lexer.advance();
            let n = match tok { Token::Number(v) => v as u32, _ => 1 };
            let _ = self.lexer.expect(&Token::RBrack)?;
            Some(n)
        } else {
            None
        };

        let kind = self.parse_constraint_kind()?;
        let mut unit = Unit::None;

        if let Token::Unit(ref u) = self.lexer.peek().0 {
            unit = u.clone();
            self.lexer.advance();
        }

        Ok(GuardDecl { name, kind, priority: Priority::Medium, unit, temporal: Temporal::None, condition: None, batch_size, span })
    }

    fn parse_constraint_kind(&mut self) -> Result<ConstraintKind> {
        if self.lexer.matches(&Token::In) {
            self.lexer.advance();
            let _ = self.lexer.expect(&Token::LBrack)?;
            let (min_tok, span) = self.lexer.advance();
            let min = match min_tok { Token::Number(n) => n, _ => return Err(CompileError::parse("expected number", span)) };
            let _ = self.lexer.expect(&Token::Comma)?;
            let (max_tok, span) = self.lexer.advance();
            let max = match max_tok { Token::Number(n) => n, _ => return Err(CompileError::parse("expected number", span)) };
            let _ = self.lexer.expect(&Token::RBrack)?;
            Ok(ConstraintKind::Range(RangeConstraint { min, max }))
        } else if self.lexer.matches(&Token::Lt) {
            self.lexer.advance();
            let (tok, span) = self.lexer.advance();
            let v = match tok { Token::Number(n) => n, _ => return Err(CompileError::parse("expected number", span)) };
            Ok(ConstraintKind::LessThan(v))
        } else if self.lexer.matches(&Token::Gt) {
            self.lexer.advance();
            let (tok, span) = self.lexer.advance();
            let v = match tok { Token::Number(n) => n, _ => return Err(CompileError::parse("expected number", span)) };
            Ok(ConstraintKind::GreaterThan(v))
        } else {
            let (_, span) = self.lexer.peek();
            Err(CompileError::parse("expected 'in', '<', or '>'", span))
        }
    }

    fn parse_priority(&mut self) -> Result<Priority> {
        let (tok, span) = self.lexer.advance();
        match tok {
            Token::High => Ok(Priority::High),
            Token::Medium => Ok(Priority::Medium),
            Token::Low => Ok(Priority::Low),
            Token::Critical => Ok(Priority::Critical),
            Token::Ident(s) if s == "CRITICAL" => Ok(Priority::Critical),
            Token::Ident(s) if s == "HIGH" => Ok(Priority::High),
            _ => Err(CompileError::parse(format!("expected priority level, got {:?}", tok), span)),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        let lhs = self.parse_atom()?;
        if self.lexer.matches(&Token::And) || self.lexer.matches(&Token::Or) {
            let op_tok = self.lexer.advance().0;
            let rhs = self.parse_atom()?;
            let op = match op_tok {
                Token::And => BinOp::And,
                Token::Or => BinOp::Or,
                _ => unreachable!(),
            };
            Ok(Expr::BinaryOp(Box::new(lhs), op, Box::new(rhs)))
        } else {
            Ok(lhs)
        }
    }

    fn parse_atom(&mut self) -> Result<Expr> {
        let (tok, _span) = self.lexer.advance();
        match tok {
            Token::Ident(s) => {
                if self.lexer.matches(&Token::Lt) || self.lexer.matches(&Token::Gt) ||
                   self.lexer.matches(&Token::Lte) || self.lexer.matches(&Token::Gte) {
                    let op_tok = self.lexer.advance().0;
                    let (rhs_tok, rhs_span) = self.lexer.advance();
                    let rhs = match rhs_tok { Token::Number(n) => n, _ => return Err(CompileError::parse("expected number", rhs_span)) };
                    let op = match op_tok {
                        Token::Lt => BinOp::Less,
                        Token::Gt => BinOp::Greater,
                        Token::Lte => BinOp::LessEq,
                        Token::Gte => BinOp::GreaterEq,
                        _ => unreachable!(),
                    };
                    Ok(Expr::BinaryOp(Box::new(Expr::Ident(s)), op, Box::new(Expr::Literal(rhs))))
                } else {
                    Ok(Expr::Ident(s))
                }
            }
            Token::Number(n) => Ok(Expr::Literal(n)),
            other => Err(CompileError::parse(format!("expected expression, got {:?}", other), Span::zero())),
        }
    }

    fn parse_rule(&mut self) -> Result<RuleDecl> {
        let _ = self.lexer.expect(&Token::Rule)?;
        let (name_tok, span) = self.lexer.advance();
        let name = match name_tok { Token::Ident(s) => s, _ => return Err(CompileError::parse("expected rule name", span)) };
        let _ = self.lexer.expect(&Token::Colon)?;

        let body = if self.lexer.matches_ident("value") {
            self.lexer.advance();
            let _ = self.lexer.expect(&Token::Clamped)?;
            let _ = self.lexer.expect(&Token::To)?;
            let _ = self.lexer.expect(&Token::LBrack)?;
            let (min_tok, sp) = self.lexer.advance();
            let min = match min_tok { Token::Number(n) => n, _ => return Err(CompileError::parse("expected number", sp)) };
            let _ = self.lexer.expect(&Token::Comma)?;
            let (max_tok, sp) = self.lexer.advance();
            let max = match max_tok { Token::Number(n) => n, _ => return Err(CompileError::parse("expected number", sp)) };
            let _ = self.lexer.expect(&Token::RBrack)?;
            RuleBody::ClampedTo { value_name: "value".into(), min, max }
        } else if self.lexer.matches_ident("max_constraints") {
            self.lexer.advance();
            let _ = self.lexer.expect(&Token::Colon)?;
            let (tok, sp) = self.lexer.advance();
            let count = match tok { Token::Number(n) => n as u32, _ => return Err(CompileError::parse("expected number", sp)) };
            let _ = self.lexer.expect(&Token::Per)?;
            let (scope_tok, sp) = self.lexer.advance();
            let scope = match scope_tok { Token::Ident(s) => s, _ => return Err(CompileError::parse("expected scope", sp)) };
            RuleBody::MaxConstraints { count, scope }
        } else {
            // severity thresholds: 0→PASS, ≤25%→CAUTION, ...
            let mut thresholds = Vec::new();
            while let Token::Number(_) = self.lexer.peek().0 {
                let (tok, _) = self.lexer.advance();
                let pct = match tok { Token::Number(n) => n, _ => break };
                if self.lexer.matches(&Token::Arrow) {
                    self.lexer.advance();
                }
                let (sev_tok, _) = self.lexer.advance();
                let sev = match sev_tok {
                    Token::Ident(s) if s == "PASS" => Severity::Pass,
                    Token::Ident(s) if s == "CAUTION" => Severity::Caution,
                    Token::Ident(s) if s == "WARNING" => Severity::Warning,
                    Token::Ident(s) if s == "CRITICAL" => Severity::Critical,
                    Token::Critical => Severity::Critical,
                    _ => Severity::Pass,
                };
                thresholds.push((pct, sev));
                if self.lexer.matches(&Token::Comma) {
                    self.lexer.advance();
                } else {
                    break;
                }
            }
            RuleBody::Severity { thresholds }
        };

        Ok(RuleDecl { name, body, span })
    }

    fn parse_preset(&mut self) -> Result<PresetDecl> {
        let _ = self.lexer.expect(&Token::Preset)?;
        let (name_tok, span) = self.lexer.advance();
        let name = match name_tok { Token::Ident(s) => s, _ => return Err(CompileError::parse("expected preset name", span)) };
        let _ = self.lexer.expect(&Token::Colon)?;

        let mut guards = Vec::new();
        while self.lexer.matches(&Token::Guard) {
            guards.push(self.parse_guard()?);
        }

        Ok(PresetDecl { name, guards, span })
    }

    fn parse_handler(&mut self) -> Result<HandlerDecl> {
        let _ = self.lexer.expect(&Token::Handle)?;
        // consume "violations"
        let _ = self.lexer.advance();
        let _ = self.lexer.expect(&Token::With)?;
        let (kind_tok, span) = self.lexer.advance();
        let kind = match kind_tok {
            Token::Log => HandlerKind::Log,
            Token::Broadcast => HandlerKind::Broadcast,
            Token::Shutdown => HandlerKind::Shutdown,
            Token::Alert => HandlerKind::Alert,
            _ => return Err(CompileError::parse("expected handler type", span)),
        };

        let condition = None;
        // skip optional WHEN clause for simplicity
        if self.lexer.matches(&Token::When) {
            self.lexer.advance();
            // consume tokens until next statement
            while !self.lexer.matches(&Token::Eof) &&
                  !self.lexer.matches(&Token::Guard) &&
                  !self.lexer.matches(&Token::Rule) &&
                  !self.lexer.matches(&Token::Preset) &&
                  !self.lexer.matches(&Token::Handle) &&
                  !self.lexer.matches(&Token::Batch) {
                self.lexer.advance();
            }
        }

        Ok(HandlerDecl { kind, condition, span })
    }
}
