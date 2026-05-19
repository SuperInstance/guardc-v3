use crate::ast::*;
use crate::error::{CompileError, Result, Span};

#[derive(Debug, Clone)]
pub struct TypedConstraint {
    pub name: String,
    pub kind: ConstraintKind,
    pub priority: Priority,
    pub unit: Unit,
    pub temporal: Temporal,
    pub batch_size: Option<u32>,
    pub typ: ConstraintType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    Range,
    Comparison,
    Boolean,
    Vectorized(usize),
}

pub fn typecheck(program: &Program) -> Result<Vec<TypedConstraint>> {
    let mut constraints = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for guard in &program.guards {
        check_guard(guard, &mut seen_names, &mut constraints)?;
    }

    for preset in &program.presets {
        for guard in &preset.guards {
            check_guard(guard, &mut seen_names, &mut constraints)?;
        }
    }

    Ok(constraints)
}

pub fn check_guard(
    guard: &GuardDecl,
    seen: &mut std::collections::HashSet<String>,
    out: &mut Vec<TypedConstraint>,
) -> Result<()> {
    if !seen.insert(guard.name.clone()) {
        return Err(CompileError::type_err(
            format!("duplicate constraint name '{}'", guard.name),
            guard.span.clone(),
        ));
    }

    // Validate range bounds
    if let ConstraintKind::Range(ref r) = guard.kind {
        if r.min > r.max {
            return Err(CompileError::type_err(
                format!("range min ({}) > max ({})", r.min, r.max),
                guard.span.clone(),
            ));
        }
    }

    if let ConstraintKind::ClampedTo(ref r) = guard.kind {
        if r.min > r.max {
            return Err(CompileError::type_err(
                format!("clamp min ({}) > max ({})", r.min, r.max),
                guard.span.clone(),
            ));
        }
    }

    // Determine type
    let typ = if guard.batch_size.is_some() {
        ConstraintType::Vectorized(guard.batch_size.unwrap() as usize)
    } else {
        match &guard.kind {
            ConstraintKind::Range(_) | ConstraintKind::ClampedTo(_) => ConstraintType::Range,
            ConstraintKind::LessThan(_) | ConstraintKind::GreaterThan(_) |
            ConstraintKind::Equal(_) | ConstraintKind::NotEqual(_) => ConstraintType::Comparison,
            ConstraintKind::And(_) | ConstraintKind::Or(_) => ConstraintType::Boolean,
        }
    };

    out.push(TypedConstraint {
        name: guard.name.clone(),
        kind: guard.kind.clone(),
        priority: guard.priority.clone(),
        unit: guard.unit.clone(),
        temporal: guard.temporal.clone(),
        batch_size: guard.batch_size,
        typ,
    });

    Ok(())
}

/// Check unit compatibility between two constraints
pub fn check_units(a: &Unit, b: &Unit, span: &Span) -> Result<()> {
    if !a.compatible(b) {
        return Err(CompileError::type_err(
            format!("incompatible units: {} vs {}", a, b),
            span.clone(),
        ));
    }
    Ok(())
}
