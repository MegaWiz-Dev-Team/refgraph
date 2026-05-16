//! Domain-specific type definitions and configurations for RefGraph
//!
//! This crate provides domain configurations for insurance, medical, legal, and finance domains.
//! Each domain defines entity types, relationship types, and consolidation rules specific to that domain.

pub mod finance;
pub mod insurance;
pub mod legal;
pub mod medical;

pub use finance::FinanceDomain;
pub use insurance::InsuranceDomain;
pub use legal::LegalDomain;
pub use medical::MedicalDomain;
