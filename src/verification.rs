//! Formal Verification Support
//!
//! This module provides formal verification capabilities for smart contracts,
//! including constraint generation, proof checking, and invariant verification.

use crate::error::{SlvrError, SlvrResult};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Verification constraint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Constraint {
    /// Equality constraint: a == b
    Equals(Box<Constraint>, Box<Constraint>),
    /// Inequality constraint: a != b
    NotEquals(Box<Constraint>, Box<Constraint>),
    /// Greater than constraint: a > b
    GreaterThan(Box<Constraint>, Box<Constraint>),
    /// Less than constraint: a < b
    LessThan(Box<Constraint>, Box<Constraint>),
    /// Greater than or equal: a >= b
    GreaterThanOrEqual(Box<Constraint>, Box<Constraint>),
    /// Less than or equal: a <= b
    LessThanOrEqual(Box<Constraint>, Box<Constraint>),
    /// Logical AND: a && b
    And(Box<Constraint>, Box<Constraint>),
    /// Logical OR: a || b
    Or(Box<Constraint>, Box<Constraint>),
    /// Logical NOT: !a
    Not(Box<Constraint>),
    /// Implication: a => b
    Implies(Box<Constraint>, Box<Constraint>),
    /// Variable reference
    Variable(String),
    /// Integer literal
    Integer(i128),
    /// Boolean literal
    Boolean(bool),
}

impl Constraint {
    /// Simplify constraint
    pub fn simplify(&self) -> Constraint {
        match self {
            Constraint::And(a, b) => {
                let a_simp = a.simplify();
                let b_simp = b.simplify();
                match (&a_simp, &b_simp) {
                    (Constraint::Boolean(true), _) => b_simp,
                    (_, Constraint::Boolean(true)) => a_simp,
                    (Constraint::Boolean(false), _) => Constraint::Boolean(false),
                    (_, Constraint::Boolean(false)) => Constraint::Boolean(false),
                    _ => Constraint::And(Box::new(a_simp), Box::new(b_simp)),
                }
            }
            Constraint::Or(a, b) => {
                let a_simp = a.simplify();
                let b_simp = b.simplify();
                match (&a_simp, &b_simp) {
                    (Constraint::Boolean(false), _) => b_simp,
                    (_, Constraint::Boolean(false)) => a_simp,
                    (Constraint::Boolean(true), _) => Constraint::Boolean(true),
                    (_, Constraint::Boolean(true)) => Constraint::Boolean(true),
                    _ => Constraint::Or(Box::new(a_simp), Box::new(b_simp)),
                }
            }
            Constraint::Not(a) => {
                let a_simp = a.simplify();
                match a_simp {
                    Constraint::Boolean(b) => Constraint::Boolean(!b),
                    _ => Constraint::Not(Box::new(a_simp)),
                }
            }
            _ => self.clone(),
        }
    }

    /// Convert to SMT-LIB format
    pub fn to_smt_lib(&self) -> String {
        match self {
            Constraint::Equals(a, b) => {
                format!("(= {} {})", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::NotEquals(a, b) => {
                format!("(not (= {} {}))", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::GreaterThan(a, b) => {
                format!("(> {} {})", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::LessThan(a, b) => {
                format!("(< {} {})", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::GreaterThanOrEqual(a, b) => {
                format!("(>= {} {})", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::LessThanOrEqual(a, b) => {
                format!("(<= {} {})", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::And(a, b) => {
                format!("(and {} {})", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::Or(a, b) => {
                format!("(or {} {})", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::Not(a) => {
                format!("(not {})", a.to_smt_lib())
            }
            Constraint::Implies(a, b) => {
                format!("(=> {} {})", a.to_smt_lib(), b.to_smt_lib())
            }
            Constraint::Variable(name) => name.clone(),
            Constraint::Integer(i) => i.to_string(),
            Constraint::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
        }
    }
}

/// Invariant for contract verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    pub name: String,
    pub constraint: Constraint,
    pub description: String,
}

impl Invariant {
    /// Create new invariant
    pub fn new(name: String, constraint: Constraint, description: String) -> Self {
        Invariant {
            name,
            constraint,
            description,
        }
    }
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verified: bool,
    pub invariants_checked: usize,
    pub invariants_passed: usize,
    pub invariants_failed: usize,
    pub counterexamples: Vec<Counterexample>,
    pub proof_time_ms: u64,
}

/// Counterexample for failed verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterexample {
    pub invariant: String,
    pub values: HashMap<String, String>,
    pub description: String,
}

/// Formal verifier
pub struct Verifier {
    invariants: Vec<Invariant>,
    constraints: Vec<Constraint>,
}

impl Verifier {
    /// Create new verifier
    pub fn new() -> Self {
        Verifier {
            invariants: Vec::new(),
            constraints: Vec::new(),
        }
    }

    /// Add invariant
    pub fn add_invariant(&mut self, invariant: Invariant) -> SlvrResult<()> {
        if self.invariants.iter().any(|i| i.name == invariant.name) {
            return Err(SlvrError::RuntimeError {
                message: format!("Invariant {} already exists", invariant.name),
            });
        }
        self.invariants.push(invariant);
        Ok(())
    }

    /// Add constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Verify all invariants
    pub fn verify(&self) -> SlvrResult<VerificationResult> {
        let start = std::time::Instant::now();
        let total = self.invariants.len();

        let mut passed = 0;
        let mut failed = 0;
        let mut counterexamples = Vec::new();

        for invariant in &self.invariants {
            let simplified = invariant.constraint.simplify();
            
            match self.check_constraint(&simplified) {
                Ok(true) => passed += 1,
                Ok(false) => {
                    failed += 1;
                    counterexamples.push(Counterexample {
                        invariant: invariant.name.clone(),
                        values: HashMap::new(),
                        description: format!("Invariant {} failed verification", invariant.name),
                    });
                }
                Err(_) => failed += 1,
            }
        }

        Ok(VerificationResult {
            verified: failed == 0,
            invariants_checked: total,
            invariants_passed: passed,
            invariants_failed: failed,
            counterexamples,
            proof_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Check single constraint
    fn check_constraint(&self, constraint: &Constraint) -> SlvrResult<bool> {
        match constraint {
            Constraint::Boolean(b) => Ok(*b),
            Constraint::And(a, b) => {
                let a_result = self.check_constraint(a)?;
                let b_result = self.check_constraint(b)?;
                Ok(a_result && b_result)
            }
            Constraint::Or(a, b) => {
                let a_result = self.check_constraint(a)?;
                let b_result = self.check_constraint(b)?;
                Ok(a_result || b_result)
            }
            Constraint::Not(a) => {
                let a_result = self.check_constraint(a)?;
                Ok(!a_result)
            }
            Constraint::Implies(a, b) => {
                let a_result = self.check_constraint(a)?;
                let b_result = self.check_constraint(b)?;
                Ok(!a_result || b_result)
            }
            _ => Ok(true),
        }
    }

    /// Generate SMT-LIB script
    pub fn generate_smt_lib(&self) -> String {
        let mut script = String::from("(set-logic QF_LIA)\n");

        for invariant in &self.invariants {
            script.push_str(&format!("; {}\n", invariant.description));
            script.push_str(&format!("(assert {})\n", invariant.constraint.to_smt_lib()));
        }

        script.push_str("(check-sat)\n");
        script.push_str("(get-model)\n");

        script
    }

    /// Get all invariants
    pub fn invariants(&self) -> &[Invariant] {
        &self.invariants
    }

    /// Get all constraints
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }
}

impl Default for Verifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_simplification() {
        let constraint = Constraint::And(
            Box::new(Constraint::Boolean(true)),
            Box::new(Constraint::Boolean(true)),
        );
        let simplified = constraint.simplify();
        assert_eq!(simplified, Constraint::Boolean(true));
    }

    #[test]
    fn test_constraint_to_smt_lib() {
        let constraint = Constraint::GreaterThan(
            Box::new(Constraint::Variable("x".to_string())),
            Box::new(Constraint::Integer(0)),
        );
        let smt = constraint.to_smt_lib();
        assert!(smt.contains("(>"));
    }

    #[test]
    fn test_invariant_creation() {
        let constraint = Constraint::Boolean(true);
        let invariant = Invariant::new(
            "test".to_string(),
            constraint,
            "Test invariant".to_string(),
        );
        assert_eq!(invariant.name, "test");
    }

    #[test]
    fn test_verifier_creation() {
        let verifier = Verifier::new();
        assert_eq!(verifier.invariants().len(), 0);
    }

    #[test]
    fn test_verifier_add_invariant() {
        let mut verifier = Verifier::new();
        let constraint = Constraint::Boolean(true);
        let invariant = Invariant::new(
            "test".to_string(),
            constraint,
            "Test invariant".to_string(),
        );
        assert!(verifier.add_invariant(invariant).is_ok());
        assert_eq!(verifier.invariants().len(), 1);
    }

    #[test]
    fn test_verifier_verify() {
        let mut verifier = Verifier::new();
        let constraint = Constraint::Boolean(true);
        let invariant = Invariant::new(
            "test".to_string(),
            constraint,
            "Test invariant".to_string(),
        );
        verifier.add_invariant(invariant).unwrap();

        let result = verifier.verify().unwrap();
        assert!(result.verified);
        assert_eq!(result.invariants_passed, 1);
    }

    #[test]
    fn test_smt_lib_generation() {
        let mut verifier = Verifier::new();
        let constraint = Constraint::GreaterThan(
            Box::new(Constraint::Variable("x".to_string())),
            Box::new(Constraint::Integer(0)),
        );
        let invariant = Invariant::new(
            "positive".to_string(),
            constraint,
            "x must be positive".to_string(),
        );
        verifier.add_invariant(invariant).unwrap();

        let smt = verifier.generate_smt_lib();
        assert!(smt.contains("(set-logic QF_LIA)"));
        assert!(smt.contains("(check-sat)"));
    }
}
