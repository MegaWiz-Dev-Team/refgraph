# RefGraph: วิธีรวมข้อมูลจากหลายแหล่งที่ยุ่งเหยิงให้เป็นระบบสะอาด

**เขียนเมื่อ:** 16 พฤษภาคม 2026  
**ผู้เขียน:** RefGraph Team  
**ติดต่อ:** paripol@megawiz.co.th  

---

## ✨ ปัญหาที่ RefGraph แก้ไข

ลองนึกภาพว่าคุณต้องรวบรวมข้อมูลประกันภัย Critical Illness (CI) จากเว็บไซต์ 3 บริษัท:

```
Prudential:  "PRU CI covers Heart Attack and Stroke"
AXA:         "AXA SIP includes coverage for myocardial infarction and CVA"
Thai Health: "Critical Care covers acute MI, cerebrovascular event"
```

**ปัญหาคืออะไร?**
- 🔄 ข้อมูลเดียวกัน อธิบายแตกต่างกัน (Heart Attack vs myocardial infarction vs acute MI)
- 📊 มี 200-300 chunks จากหลายแหล่งที่ทับซ้อนกัน
- ⏰ ถ้ารวมด้วยมือ = เสียเวลา + ผิดพลาด
- 💾 ไม่รู้ว่าข้อมูลมาจากไหน

---

## 🎯 RefGraph ช่วยได้ยังไง?

RefGraph จะ:

1. **ดึงเอา entities** จากข้อความ
   - Heart Attack = ประกาศโครงสร้าง (coverage)
   - Pre-existing condition = ยกเว้น (exclusion)

2. **สร้างกราฟ** ของความสัมพันธ์
   ```
   Critical Illness --[HAS_COVERAGE]--> Heart Attack
   Critical Illness --[HAS_EXCLUSION]--> Pre-existing
   ```

3. **รวม (Deduplicate)** ข้อมูลซ้ำ
   - "Heart Attack" = "myocardial infarction" = "acute MI" → 1 entity

4. **ทำให้สะอาด** ด้วยจุดที่เชื่อถือได้
   - confidence = 0.88 (ว่ามั่นแค่ไหน)
   - source = ["prudential.co.th", "axa.co.th"] (มาจากไหนบ้าง)

**ผลลัพธ์:** จาก 1200+ chunks ยุ่งเหยิง → 900 entities สะอาด พร้อมใช้ได้

---

## 🚀 ลองใช้ RefGraph

### ขั้นที่ 1: ติดตั้ง

```toml
[dependencies]
refgraph-core = "0.1"
tokio = { version = "1.49", features = ["full"] }
```

### ขั้นที่ 2: โค้ดตัวอย่าง

```rust
use refgraph_core::{RefGraph, ManifestConfig, types::RawChunk};

#[tokio::main]
async fn main() -> refgraph_core::Result<()> {
    // เลือก domain (ประเภท: ประกันภัย, การแพทย์, กฎหมาย, การเงิน)
    let config = ManifestConfig::insurance();
    
    // สร้าง pipeline
    let mut graph = RefGraph::new(config)?;
    
    // โหลดข้อมูลจากเว็บ
    let chunks = vec![
        RawChunk {
            chunk_id: "pru_001".to_string(),
            content: "PRU CI covers Heart Attack and Stroke".to_string(),
            source_url: "prudential.co.th/products".to_string(),
            page_index: Some(1),
            token_count: 10,
        },
        // เพิ่มข้อมูลเพิ่มเติม...
    ];
    
    // รัน consolidation
    let output = graph.consolidate(chunks).await?;
    
    // ดูผลลัพธ์
    println!("✓ {} entities", output.metadata.entity_count);
    println!("✓ {} relationships", output.metadata.relationship_count);
    
    // บันทึกเป็น JSON
    output.save_json("output.json")?;
    
    Ok(())
}
```

### ขั้นที่ 3: ตั้งค่า Domain

RefGraph ทำงานกับ 4 domains:

**1. Insurance (ประกันภัย)** - Default
```toml
[domain]
name = "insurance"

[entities.product]
patterns = ["Critical Illness", "Health Insurance", "Life Insurance"]

[entities.coverage]
patterns = ["Heart Attack", "Stroke", "Cancer"]

[entities.exclusion]
patterns = ["Pre-existing Condition", "War"]
```

**2. Medical (การแพทย์)**
```toml
[domain]
name = "medical"

[entities.symptom]
patterns = ["fever", "cough", "dyspnea"]

[entities.drug]
patterns = ["metformin", "aspirin", "atorvastatin"]
```

**3. Legal (กฎหมาย)**
```toml
[domain]
name = "legal"

[entities.clause]
patterns = ["clause", "provision", "section"]

[entities.obligation]
patterns = ["shall", "must", "required"]
```

**4. Finance (การเงิน)**
```toml
[domain]
name = "finance"

[entities.fee]
patterns = ["fee", "charge", "commission"]

[entities.rate]
patterns = ["interest rate", "APR", "APY"]
```

---

## 🔄 Pipeline ทำงานอย่างไร

```
┌──────────────────┐
│  200-300 Chunks  │  ข้อมูลดิบจากเว็บ
└─────────┬────────┘
          │
    ┌─────▼──────┐
    │ [1] EXTRACT │  ดึง entities จากข้อความ
    └─────┬──────┘
          │
    ┌─────▼──────┐
    │ [2] GRAPH  │  สร้างความสัมพันธ์
    └─────┬──────┘
          │
    ┌─────▼──────┐
    │ [3] DEDUP  │  รวมข้อมูลซ้ำ (Jaccard 0.95)
    └─────┬──────┘
          │
    ┌─────▼───────────┐
    │ [4] CONSOLIDATE │  รวมแบบ semantic (cosine 0.96)
    └─────┬───────────┘
          │
    ┌─────▼────────────────┐
    │ 850-950 Clean Entities│  ผลลัพธ์สะอาด + ความสัมพันธ์
    └──────────────────────┘
```

---

## 💡 ตัวอย่างจริง: ประกันภัย

**Input:** ข้อมูล 5 chunks จาก 3 บริษัท

```json
[
  {"id": "pru_1", "text": "PRU Critical Illness covers Heart Attack and Stroke", "source": "prudential.co.th"},
  {"id": "pru_2", "text": "Pre-existing conditions excluded", "source": "prudential.co.th"},
  {"id": "axa_1", "text": "AXA SIP includes coverage for myocardial infarction and CVA", "source": "axa.co.th"},
  {"id": "axa_2", "text": "AXA excludes pre-existing medical conditions", "source": "axa.co.th"},
  {"id": "th_1", "text": "Thai Health Critical Care covers acute MI and cerebrovascular event", "source": "thaihealth.co.th"}
]
```

**Output:** Consolidated entities

```json
{
  "entities": [
    {
      "text": "Critical Illness",
      "type": "product",
      "confidence": 0.92,
      "sources": ["prudential.co.th", "axa.co.th"],
      "merged_from": ["prod_critical_illness"]
    },
    {
      "text": "Heart Attack",
      "type": "coverage",
      "confidence": 0.88,
      "sources": ["prudential.co.th", "axa.co.th", "thaihealth.co.th"],
      "merged_from": ["cov_heart_attack", "cov_myocardial_infarction", "cov_acute_mi"]
    },
    {
      "text": "Pre-existing Condition",
      "type": "exclusion",
      "confidence": 0.85,
      "sources": ["prudential.co.th", "axa.co.th"]
    }
  ],
  "relationships": [
    {
      "source": "prod_critical_illness",
      "target": "cov_heart_attack",
      "type": "HAS_COVERAGE",
      "confidence": 0.90
    }
  ]
}
```

**ข้อดี:**
- ✅ Heart Attack, MI, acute MI → รวมเป็น 1 entity
- ✅ รู้ว่า Critical Illness → HAS_COVERAGE → Heart Attack
- ✅ เก็บ sources ทั้งหมด (ความโปร่งใส)
- ✅ confidence scores (วัดความมั่นใจ)

---

## 🎁 RefGraph เหมาะสำหรับ

### 1. **ธุรกิจประกันภัย**
- รวมผลิตภัณฑ์จากคู่แข่ง
- สร้างเปรียบเทียบ (comparison)
- ตรวจสอบการเปลี่ยนแปลงเงื่อนไข

### 2. **ห้องเรียน/วิจัย**
- รวมข้อมูลจากบทความวิทยาศาสตร์
- สร้างกราฟ concept
- ค้นหา semantic relationships

### 3. **Legal Tech**
- สกัด clauses จากสัญญา
- เปรียบเทียบเงื่อนไขกฎหมาย
- ตรวจสอบความขัดแย้ง

### 4. **FinTech**
- รวม fee structures จากธนาคาร
- เปรียบเทียบอัตราดอกเบี้ย
- สร้าง product comparison engine

---

## 📊 Performance

```
100 chunks:
  ✓ Extract: 50ms
  ✓ Dedup: 30ms
  ✓ Graph: 20ms
  = ~100ms total

1000 chunks:
  ✓ Extract: 500ms
  ✓ Dedup: 300ms
  ✓ Graph: 200ms
  = ~1 second total
```

---

## 🔐 License

RefGraph ใช้ **Apache 2.0** - ฟรี! ใช้ได้ตามใจชอบ

```
✅ ใช้เพื่อผลกำไร
✅ แก้ไขโค้ด
✅ ใช้ใน SaaS
✅ ขายต่อ
```

ไม่ต้องเปิด source code ของคุณ

---

## 🚀 เริ่มใช้ตอนนี้

```bash
# 1. Clone
git clone https://github.com/refgraph-rs/refgraph.git

# 2. Run example
cargo run -p refgraph-core --example insurance_pipeline

# 3. Add to project
cargo add refgraph-core
```

---

## 📧 ติดต่อ

- **GitHub:** https://github.com/refgraph-rs/refgraph
- **Crates.io:** https://crates.io/crates/refgraph-core
- **Email:** paripol@megawiz.co.th

---

**RefGraph:** ทำให้ข้อมูลยุ่งเหยิงกลายเป็นสมบูรณ์ 🎯

