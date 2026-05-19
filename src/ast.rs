use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "LOW"),
            Priority::Medium => write!(f, "MEDIUM"),
            Priority::High => write!(f, "HIGH"),
            Priority::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Severity {
    Pass,
    Caution,
    Warning,
    Critical,
}

impl Severity {
    pub fn from_pct(pct: f64) -> Self {
        if pct == 0.0 { Severity::Pass }
        else if pct <= 25.0 { Severity::Caution }
        else if pct <= 50.0 { Severity::Warning }
        else { Severity::Critical }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Unit {
    None,
    Celsius,
    Fahrenheit,
    Kelvin,
    KPa,
    MPa,
    PSI,
    Percent,
    Meters,
    Seconds,
    PerSecond,
    Volts,
    Amperes,
    Watts,
    RPM,
    Custom(String),
}

impl Unit {
    pub fn compatible(&self, other: &Unit) -> bool {
        match (self, other) {
            (Unit::None, _) | (_, Unit::None) => true,
            (Unit::Celsius, Unit::Celsius) => true,
            (Unit::Celsius, Unit::Fahrenheit) => true,
            (Unit::Celsius, Unit::Kelvin) => true,
            (Unit::Fahrenheit, Unit::Fahrenheit) => true,
            (Unit::Fahrenheit, Unit::Celsius) => true,
            (Unit::Fahrenheit, Unit::Kelvin) => true,
            (Unit::Kelvin, Unit::Kelvin) => true,
            (Unit::Kelvin, Unit::Celsius) => true,
            (Unit::Kelvin, Unit::Fahrenheit) => true,
            (Unit::KPa, Unit::KPa) => true,
            (Unit::KPa, Unit::MPa) => true,
            (Unit::KPa, Unit::PSI) => true,
            (Unit::MPa, Unit::MPa) => true,
            (Unit::MPa, Unit::KPa) => true,
            (Unit::PSI, Unit::PSI) => true,
            (Unit::PSI, Unit::KPa) => true,
            (Unit::Percent, Unit::Percent) => true,
            (Unit::Meters, Unit::Meters) => true,
            (Unit::Seconds, Unit::Seconds) => true,
            (Unit::PerSecond, Unit::PerSecond) => true,
            (Unit::Volts, Unit::Volts) => true,
            (Unit::Amperes, Unit::Amperes) => true,
            (Unit::Watts, Unit::Watts) => true,
            (Unit::RPM, Unit::RPM) => true,
            (Unit::Custom(a), Unit::Custom(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::None => write!(f, ""),
            Unit::Celsius => write!(f, "°C"),
            Unit::Fahrenheit => write!(f, "°F"),
            Unit::Kelvin => write!(f, "K"),
            Unit::KPa => write!(f, "kPa"),
            Unit::MPa => write!(f, "MPa"),
            Unit::PSI => write!(f, "PSI"),
            Unit::Percent => write!(f, "%"),
            Unit::Meters => write!(f, "m"),
            Unit::Seconds => write!(f, "s"),
            Unit::PerSecond => write!(f, "/s"),
            Unit::Volts => write!(f, "V"),
            Unit::Amperes => write!(f, "A"),
            Unit::Watts => write!(f, "W"),
            Unit::RPM => write!(f, "RPM"),
            Unit::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RangeConstraint {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone)]
pub enum ConstraintKind {
    Range(RangeConstraint),
    LessThan(f64),
    GreaterThan(f64),
    Equal(f64),
    NotEqual(f64),
    And(Vec<ConstraintKind>),
    Or(Vec<ConstraintKind>),
    ClampedTo(RangeConstraint),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(String),
    Literal(f64),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    And,
    Or,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
pub enum Temporal {
    None,
    PerSecond,
    PerMinute,
    ForSeconds(u64),
    ForMinutes(u64),
}

#[derive(Debug, Clone)]
pub struct GuardDecl {
    pub name: String,
    pub kind: ConstraintKind,
    pub priority: Priority,
    pub unit: Unit,
    pub temporal: Temporal,
    pub condition: Option<Expr>,
    pub batch_size: Option<u32>,
    pub span: crate::error::Span,
}

#[derive(Debug, Clone)]
pub struct RuleDecl {
    pub name: String,
    pub body: RuleBody,
    pub span: crate::error::Span,
}

#[derive(Debug, Clone)]
pub enum RuleBody {
    ClampedTo { value_name: String, min: f64, max: f64 },
    MaxConstraints { count: u32, scope: String },
    Severity { thresholds: Vec<(f64, Severity)> },
}

#[derive(Debug, Clone)]
pub struct PresetDecl {
    pub name: String,
    pub guards: Vec<GuardDecl>,
    pub span: crate::error::Span,
}

#[derive(Debug, Clone)]
pub enum HandlerKind {
    Log,
    Broadcast,
    Shutdown,
    Alert,
}

#[derive(Debug, Clone)]
pub struct HandlerDecl {
    pub kind: HandlerKind,
    pub condition: Option<String>,
    pub span: crate::error::Span,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub guards: Vec<GuardDecl>,
    pub rules: Vec<RuleDecl>,
    pub presets: Vec<PresetDecl>,
    pub handlers: Vec<HandlerDecl>,
}
