//! Insurance Company Data Loaders
//! Provides sample and realistic insurance documents for S1 execution

use crate::reference_implementation::{ConsolidatedEntity, ConsolidatedRelationship, ConsolidationOutput};
use serde::{Deserialize, Serialize};

/// Insurance company identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsuranceCompany {
    Prudential,
    AXA,
    ThaiHealth,
}

impl InsuranceCompany {
    pub fn as_str(&self) -> &'static str {
        match self {
            InsuranceCompany::Prudential => "Prudential",
            InsuranceCompany::AXA => "AXA",
            InsuranceCompany::ThaiHealth => "Thai Health",
        }
    }
}

/// Insurance document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceDocument {
    pub id: String,
    pub company: InsuranceCompany,
    pub policy_type: String,
    pub content: String,
    pub language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Thai,
    Mixed,
}

impl InsuranceDocument {
    pub fn new(
        id: String,
        company: InsuranceCompany,
        policy_type: String,
        content: String,
        language: Language,
    ) -> Self {
        Self {
            id,
            company,
            policy_type,
            content,
            language,
        }
    }
}

/// Prudential Insurance Documents
pub mod prudential {
    use super::*;

    pub fn critical_illness_policy() -> InsuranceDocument {
        InsuranceDocument::new(
            "PRUD_CI_001".to_string(),
            InsuranceCompany::Prudential,
            "Critical Illness".to_string(),
            "PRUDENTIAL CRITICAL ILLNESS INSURANCE\n\
             \n\
             Coverage includes:\n\
             - Heart Attack with 4 week survival\n\
             - Stroke with permanent neurological damage\n\
             - Cancer with histological confirmation\n\
             - Kidney Failure requiring dialysis\n\
             - Liver Failure with cirrhosis\n\
             - Paralysis from spinal cord injury\n\
             \n\
             Exclusions:\n\
             - Pre-existing conditions (18 month wait)\n\
             - Alcohol/drug related illnesses\n\
             - High-risk hobbies\n\
             - War or civil unrest\n\
             \n\
             Benefit Amount: Up to $250,000\n\
             Premium: Monthly or Annual payment available"
                .to_string(),
            Language::English,
        )
    }

    pub fn life_insurance_policy() -> InsuranceDocument {
        InsuranceDocument::new(
            "PRUD_LIFE_001".to_string(),
            InsuranceCompany::Prudential,
            "Life Insurance".to_string(),
            "PRUDENTIAL LIFE INSURANCE PLAN\n\
             \n\
             Death Benefit Coverage:\n\
             - Natural death: 100% of sum assured\n\
             - Accidental death: 200% of sum assured\n\
             - Terminal illness: 50% advance payment\n\
             \n\
             Riders Available:\n\
             - Accidental Death & Disability (AD&D)\n\
             - Waiver of Premium\n\
             - Critical Illness rider\n\
             \n\
             Claim Settlement:\n\
             - Submit claim within 90 days\n\
             - Documentation requirements listed\n\
             - Settlement within 30 days of approval\n\
             \n\
             Premium Payment: Monthly, Quarterly, Annual"
                .to_string(),
            Language::English,
        )
    }

    pub fn health_insurance_policy() -> InsuranceDocument {
        InsuranceDocument::new(
            "PRUD_HEALTH_001".to_string(),
            InsuranceCompany::Prudential,
            "Health Insurance".to_string(),
            "PRUDENTIAL COMPREHENSIVE HEALTH INSURANCE\n\
             \n\
             Coverage includes:\n\
             - Hospital room and board\n\
             - Surgical procedures\n\
             - ICU charges\n\
             - Doctor consultation fees\n\
             - Diagnostic tests and imaging\n\
             - Medications and supplies\n\
             - Nursing care\n\
             \n\
             Daily Hospital Benefit: Up to $500/day\n\
             Surgical Benefit: Up to $50,000\n\
             ICU Benefit: Up to $1,000/day\n\
             \n\
             Claim Period: 12 months\n\
             Waiting Period: 30 days (general), 6 months (pre-existing)\n\
             \n\
             Network Hospitals: 5000+ across region"
                .to_string(),
            Language::English,
        )
    }
}

/// AXA Insurance Documents
pub mod axa {
    use super::*;

    pub fn travel_insurance_policy() -> InsuranceDocument {
        InsuranceDocument::new(
            "AXA_TRAVEL_001".to_string(),
            InsuranceCompany::AXA,
            "Travel Insurance".to_string(),
            "AXA COMPREHENSIVE TRAVEL INSURANCE\n\
             \n\
             Trip Coverage:\n\
             - Medical emergency abroad\n\
             - Emergency evacuation\n\
             - Trip cancellation\n\
             - Baggage loss/delay\n\
             - Travel delay\n\
             \n\
             Medical Coverage Limit: $100,000\n\
             Trip Cancellation: Up to trip cost\n\
             Baggage Coverage: Up to $5,000\n\
             \n\
             Coverage Territory:\n\
             - Worldwide coverage available\n\
             - Includes high-risk countries (with premium)\n\
             - 24/7 assistance hotline\n\
             \n\
             Valid Duration: 1-365 days\n\
             Annual policies available"
                .to_string(),
            Language::English,
        )
    }

    pub fn auto_insurance_policy() -> InsuranceDocument {
        InsuranceDocument::new(
            "AXA_AUTO_001".to_string(),
            InsuranceCompany::AXA,
            "Auto Insurance".to_string(),
            "AXA COMPREHENSIVE AUTO INSURANCE\n\
             \n\
             Coverage Options:\n\
             - Liability coverage (bodily injury & property)\n\
             - Collision coverage\n\
             - Comprehensive coverage\n\
             - Uninsured motorist coverage\n\
             - Roadside assistance\n\
             \n\
             Liability Limits:\n\
             - Bodily injury: $50,000 per person / $100,000 per accident\n\
             - Property damage: $25,000\n\
             \n\
             Deductibles: $250, $500, $1,000 options\n\
             Annual Policy Period\n\
             \n\
             Additional Benefits:\n\
             - Rental car coverage\n\
             - Towing and labor\n\
             - 24/7 claims support"
                .to_string(),
            Language::English,
        )
    }

    pub fn home_insurance_policy() -> InsuranceDocument {
        InsuranceDocument::new(
            "AXA_HOME_001".to_string(),
            InsuranceCompany::AXA,
            "Home Insurance".to_string(),
            "AXA HOME INSURANCE POLICY\n\
             \n\
             Dwelling Coverage:\n\
             - Building structure and fixtures\n\
             - Extensions and outbuildings\n\
             - Loss of use (temporary housing)\n\
             \n\
             Personal Property Coverage:\n\
             - Furniture and belongings\n\
             - Electronics and valuables\n\
             - Temporary storage coverage\n\
             \n\
             Liability Coverage: Up to $500,000\n\
             Medical Payments: $1,000 per occurrence\n\
             \n\
             Deductible Options: $500 - $2,500\n\
             Special Coverage:\n\
             - Theft and burglary\n\
             - Natural disasters\n\
             - Accidental damage (optional)\n\
             \n\
             Claims: 24/7 emergency hotline"
                .to_string(),
            Language::English,
        )
    }
}

/// Thai Health Insurance Documents
pub mod thai_health {
    use super::*;

    pub fn thai_health_comprehensive() -> InsuranceDocument {
        InsuranceDocument::new(
            "TH_COMP_001".to_string(),
            InsuranceCompany::ThaiHealth,
            "Comprehensive Health".to_string(),
            "ประกันสุขภาพแบบครอบคลุม (Comprehensive Health Insurance)\n\n\
             การครอบคลุม (Coverage):\n\
             - การเข้ารับการรักษาที่โรงพยาบาล (Hospital admission)\n\
             - ค่าห้องพยาบาล (Room charges)\n\
             - ค่าหมอและการผ่าตัด (Doctor and surgical fees)\n\
             - ค่ายาและวัสดุ (Medications and supplies)\n\
             - ส่วนลดทันตกรรม (Dental discount)\n\
             - ตรวจสุขภาพ (Health screening)\n\
             \n\
             วงเงินประกัน (Coverage Limits):\n\
             - ห้องพยาบาล: ไม่มีข้อจำกัด (Unlimited room)\n\
             - ประกอบการการแพทย์: 5,000,000 บาท\n\
             - โรคประจำตัว: 1,000,000 บาท\n\
             \n\
             ข้อยกเว้น (Exclusions):\n\
             - โรคประจำตัว (Pre-existing conditions)\n\
             - การกำเนิด (Pregnancy)\n\
             - พฤติกรรมเสี่ยง (High-risk activities)"
                .to_string(),
            Language::Thai,
        )
    }

    pub fn thai_health_life_insurance() -> InsuranceDocument {
        InsuranceDocument::new(
            "TH_LIFE_001".to_string(),
            InsuranceCompany::ThaiHealth,
            "Life Insurance".to_string(),
            "ประกันชีวิต Thai Health (Thai Health Life Insurance)\n\n\
             สิทธิประโยชน์ (Benefits):\n\
             - เงินประกันอุบัติเหตุ (Accidental death): 200% ของเงินประกัน\n\
             - เงินประกันเสียชีวิตตามธรรมชาติ (Natural death): 100%\n\
             - เงินสำหรับความพิการถาวร (Disability): 100%\n\
             - บำนาญชีวิตให้แก่บุตร (Children's benefit)\n\
             \n\
             ระยะเวลาการประกัน (Term):\n\
             - ประกันตลอดชีวิต (Whole life)\n\
             - ประกันระยะเวลา 20/30 ปี (Term policies)\n\
             \n\
             การจ่ายเบี้ยประกัน (Premium Payment):\n\
             - รายเดือน (Monthly)\n\
             - รายปี (Annual)\n\
             - เพื่อชีวิต (Single premium)\n\
             \n\
             สิทธิพิเศษ (Riders):\n\
             - การรักษาพยาบาลวิกฤต (Critical illness)\n\
             - วิจารณญาณพิการ (Disability waiver)"
                .to_string(),
            Language::Thai,
        )
    }

    pub fn thai_health_accident() -> InsuranceDocument {
        InsuranceDocument::new(
            "TH_ACC_001".to_string(),
            InsuranceCompany::ThaiHealth,
            "Accident Insurance".to_string(),
            "ประกันอุบัติเหตุ (Accident Insurance)\n\n\
             ความคุ้มครอง (Coverage):\n\
             - เสียชีวิตจากอุบัติเหตุ (Accidental death)\n\
             - อุบัติเหตุที่ส่งผลให้เป็นพิการถาวร (Permanent disability)\n\
             - ค่ารักษาพยาบาลจากอุบัติเหตุ (Medical treatment)\n\
             - ค่าวันพักรักษา (Hospitalization benefit)\n\
             \n\
             ข้อกำหนด (Terms):\n\
             - อายุขององค์ประกอบ: 18-65 ปี (Age limit)\n\
             - ระยะเวลา: 1-2 ปี (Coverage period)\n\
             - อัตราเบี้ย: ประมาณ 500 บาท/ปี (Annual premium)\n\
             \n\
             การจ่ายเบี้ยประกัน:\n\
             - เงินสด (Cash payment)\n\
             - ธนาคาร (Bank transfer)\n\
             - เช็ค (Check)\n\
             \n\
             ข้อยกเว้น:\n\
             - กีฬาอันตราย (Dangerous sports)\n\
             - การขับขี่ที่ประมาท (Reckless driving)"
                .to_string(),
            Language::Thai,
        )
    }
}

/// Load all sample documents for a given company
pub fn load_company_documents(company: InsuranceCompany) -> Vec<InsuranceDocument> {
    match company {
        InsuranceCompany::Prudential => vec![
            prudential::critical_illness_policy(),
            prudential::life_insurance_policy(),
            prudential::health_insurance_policy(),
        ],
        InsuranceCompany::AXA => vec![
            axa::travel_insurance_policy(),
            axa::auto_insurance_policy(),
            axa::home_insurance_policy(),
        ],
        InsuranceCompany::ThaiHealth => vec![
            thai_health::thai_health_comprehensive(),
            thai_health::thai_health_life_insurance(),
            thai_health::thai_health_accident(),
        ],
    }
}

/// Generate sample consolidation for a company
pub fn generate_consolidation_for_company(
    company: InsuranceCompany,
) -> ConsolidationOutput {
    let documents = load_company_documents(company);

    match company {
        InsuranceCompany::Prudential => ConsolidationOutput {
            domain: "insurance".to_string(),
            entities: vec![
                ConsolidatedEntity {
                    id: "ent_prud_001".to_string(),
                    entity_type: "COMPANY".to_string(),
                    text: "Prudential".to_string(),
                    confidence: 0.99,
                    sources: vec!["PRUD_CI_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_prud_002".to_string(),
                    entity_type: "PRODUCT".to_string(),
                    text: "Critical Illness Insurance".to_string(),
                    confidence: 0.96,
                    sources: vec!["PRUD_CI_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_prud_003".to_string(),
                    entity_type: "COVERAGE".to_string(),
                    text: "Heart Attack".to_string(),
                    confidence: 0.94,
                    sources: vec!["PRUD_CI_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_prud_004".to_string(),
                    entity_type: "BENEFIT".to_string(),
                    text: "Death Benefit".to_string(),
                    confidence: 0.95,
                    sources: vec!["PRUD_LIFE_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_prud_005".to_string(),
                    entity_type: "EXCLUSION".to_string(),
                    text: "Pre-existing conditions".to_string(),
                    confidence: 0.92,
                    sources: vec!["PRUD_CI_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_prud_006".to_string(),
                    entity_type: "COVERAGE".to_string(),
                    text: "Hospital admission".to_string(),
                    confidence: 0.94,
                    sources: vec!["PRUD_HEALTH_001".to_string()],
                },
            ],
            relationships: vec![
                ConsolidatedRelationship {
                    source_id: "ent_prud_001".to_string(),
                    target_id: "ent_prud_002".to_string(),
                    relationship_type: "OFFERS_PRODUCT".to_string(),
                    confidence: 0.98,
                },
                ConsolidatedRelationship {
                    source_id: "ent_prud_002".to_string(),
                    target_id: "ent_prud_003".to_string(),
                    relationship_type: "HAS_COVERAGE".to_string(),
                    confidence: 0.96,
                },
                ConsolidatedRelationship {
                    source_id: "ent_prud_002".to_string(),
                    target_id: "ent_prud_005".to_string(),
                    relationship_type: "HAS_EXCLUSION".to_string(),
                    confidence: 0.93,
                },
                ConsolidatedRelationship {
                    source_id: "ent_prud_004".to_string(),
                    target_id: "ent_prud_001".to_string(),
                    relationship_type: "PROVIDED_BY".to_string(),
                    confidence: 0.95,
                },
            ],
            average_confidence: 0.945,
        },
        InsuranceCompany::AXA => ConsolidationOutput {
            domain: "insurance".to_string(),
            entities: vec![
                ConsolidatedEntity {
                    id: "ent_axa_001".to_string(),
                    entity_type: "COMPANY".to_string(),
                    text: "AXA".to_string(),
                    confidence: 0.99,
                    sources: vec!["AXA_TRAVEL_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_axa_002".to_string(),
                    entity_type: "PRODUCT".to_string(),
                    text: "Travel Insurance".to_string(),
                    confidence: 0.95,
                    sources: vec!["AXA_TRAVEL_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_axa_003".to_string(),
                    entity_type: "COVERAGE".to_string(),
                    text: "Medical emergency abroad".to_string(),
                    confidence: 0.93,
                    sources: vec!["AXA_TRAVEL_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_axa_004".to_string(),
                    entity_type: "PRODUCT".to_string(),
                    text: "Auto Insurance".to_string(),
                    confidence: 0.96,
                    sources: vec!["AXA_AUTO_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_axa_005".to_string(),
                    entity_type: "BENEFIT".to_string(),
                    text: "Roadside assistance".to_string(),
                    confidence: 0.94,
                    sources: vec!["AXA_AUTO_001".to_string()],
                },
            ],
            relationships: vec![
                ConsolidatedRelationship {
                    source_id: "ent_axa_001".to_string(),
                    target_id: "ent_axa_002".to_string(),
                    relationship_type: "OFFERS_PRODUCT".to_string(),
                    confidence: 0.98,
                },
                ConsolidatedRelationship {
                    source_id: "ent_axa_002".to_string(),
                    target_id: "ent_axa_003".to_string(),
                    relationship_type: "HAS_COVERAGE".to_string(),
                    confidence: 0.95,
                },
                ConsolidatedRelationship {
                    source_id: "ent_axa_001".to_string(),
                    target_id: "ent_axa_004".to_string(),
                    relationship_type: "OFFERS_PRODUCT".to_string(),
                    confidence: 0.97,
                },
            ],
            average_confidence: 0.953,
        },
        InsuranceCompany::ThaiHealth => ConsolidationOutput {
            domain: "insurance".to_string(),
            entities: vec![
                ConsolidatedEntity {
                    id: "ent_th_001".to_string(),
                    entity_type: "COMPANY".to_string(),
                    text: "Thai Health".to_string(),
                    confidence: 0.99,
                    sources: vec!["TH_COMP_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_th_002".to_string(),
                    entity_type: "PRODUCT".to_string(),
                    text: "ประกันสุขภาพแบบครอบคลุม".to_string(),
                    confidence: 0.96,
                    sources: vec!["TH_COMP_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_th_003".to_string(),
                    entity_type: "COVERAGE".to_string(),
                    text: "Hospital admission".to_string(),
                    confidence: 0.94,
                    sources: vec!["TH_COMP_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_th_004".to_string(),
                    entity_type: "PRODUCT".to_string(),
                    text: "ประกันชีวิต".to_string(),
                    confidence: 0.95,
                    sources: vec!["TH_LIFE_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_th_005".to_string(),
                    entity_type: "BENEFIT".to_string(),
                    text: "Death Benefit".to_string(),
                    confidence: 0.93,
                    sources: vec!["TH_LIFE_001".to_string()],
                },
            ],
            relationships: vec![
                ConsolidatedRelationship {
                    source_id: "ent_th_001".to_string(),
                    target_id: "ent_th_002".to_string(),
                    relationship_type: "OFFERS_PRODUCT".to_string(),
                    confidence: 0.97,
                },
                ConsolidatedRelationship {
                    source_id: "ent_th_002".to_string(),
                    target_id: "ent_th_003".to_string(),
                    relationship_type: "HAS_COVERAGE".to_string(),
                    confidence: 0.95,
                },
                ConsolidatedRelationship {
                    source_id: "ent_th_001".to_string(),
                    target_id: "ent_th_004".to_string(),
                    relationship_type: "OFFERS_PRODUCT".to_string(),
                    confidence: 0.96,
                },
            ],
            average_confidence: 0.947,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prudential_documents_loaded() {
        let docs = load_company_documents(InsuranceCompany::Prudential);
        assert_eq!(docs.len(), 3);
        assert_eq!(docs[0].company, InsuranceCompany::Prudential);
    }

    #[test]
    fn test_axa_documents_loaded() {
        let docs = load_company_documents(InsuranceCompany::AXA);
        assert_eq!(docs.len(), 3);
    }

    #[test]
    fn test_thai_health_documents_loaded() {
        let docs = load_company_documents(InsuranceCompany::ThaiHealth);
        assert_eq!(docs.len(), 3);
    }

    #[test]
    fn test_prudential_consolidation_generated() {
        let consolidation = generate_consolidation_for_company(InsuranceCompany::Prudential);
        assert_eq!(consolidation.entities.len(), 6);
        assert_eq!(consolidation.relationships.len(), 4);
        assert!(consolidation.average_confidence > 0.93);
    }

    #[test]
    fn test_axa_consolidation_generated() {
        let consolidation = generate_consolidation_for_company(InsuranceCompany::AXA);
        assert_eq!(consolidation.entities.len(), 5);
        assert_eq!(consolidation.relationships.len(), 3);
    }

    #[test]
    fn test_thai_health_consolidation_generated() {
        let consolidation = generate_consolidation_for_company(InsuranceCompany::ThaiHealth);
        assert_eq!(consolidation.entities.len(), 5);
        assert_eq!(consolidation.relationships.len(), 3);
    }

    #[test]
    fn test_thai_documents_contain_thai_text() {
        let docs = load_company_documents(InsuranceCompany::ThaiHealth);
        assert!(docs.iter().any(|d| d.content.contains("ประกัน")));
        assert!(docs.iter().any(|d| d.language == Language::Thai));
    }

    #[test]
    fn test_insurance_company_as_str() {
        assert_eq!(InsuranceCompany::Prudential.as_str(), "Prudential");
        assert_eq!(InsuranceCompany::AXA.as_str(), "AXA");
        assert_eq!(InsuranceCompany::ThaiHealth.as_str(), "Thai Health");
    }
}
