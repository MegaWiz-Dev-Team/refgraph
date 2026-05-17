# การลงทะเบียน OAuth2 Client สำหรับ Mimir

## ข้อมูลที่ต้องใช้

```
Client ID:     368876822349333070
Client Secret: CeGcfzog8DcJ4cxK658tRtCeEgFwdt1AAU60twz5Su1qIpsGtfdqtq8NYAryxJzmU
Yggdrasil URL: http://localhost:8085
```

## วิธีการ (เลือกหนึ่งในสี่วิธี)

### ❌ วิธี A: UI Admin (ต้องรู้ password admin)

1. เปิด http://localhost:8085/ui/login
2. Login ด้วย admin credentials
3. ไปที่ Clients / Applications menu
4. สร้างใหม่ (Create):
   - Name: `mimir-api`
   - Client ID: `368876822349333070`
   - Client Secret: `CeGcfzog8DcJ4cxK658tRtCeEgFwdt1AAU60twz5Su1qIpsGtfdqtq8NYAryxJzmU`
   - Grant Type: `client_credentials`
   - Scopes: `openid profile email`

### ✅ วิธี B: SQL Query (ถ้ามี DB access)

```bash
# Port-forward to MariaDB
kubectl port-forward -n asgard-infra service/mariadb 3306:3306 &

# Connect and register
mysql -h 127.0.0.1 -u mimir -p'mimir_password' yggdrasil << 'SQL'
INSERT INTO oauth2_clients (
  client_id,
  client_secret,
  client_name,
  grant_types,
  scopes,
  created_at
) VALUES (
  '368876822349333070',
  'CeGcfzog8DcJ4cxK658tRtCeEgFwdt1AAU60twz5Su1qIpsGtfdqtq8NYAryxJzmU',
  'mimir-api',
  'client_credentials',
  'openid profile email',
  NOW()
);
SQL

echo "Client registered!"
```

### ✅ วิธี C: API Endpoint (ถ้ามี master token)

```bash
MASTERKEY=$(kubectl get secret asgard-secrets -n asgard -o jsonpath='{.data.YGGDRASIL_MASTERKEY}' | base64 -d)

curl -s -X POST http://localhost:8085/admin/oauth2/clients \
  -H "X-Master-Key: $MASTERKEY" \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "368876822349333070",
    "client_secret": "CeGcfzog8DcJ4cxK658tRtCeEgFwdt1AAU60twz5Su1qIpsGtfdqtq8NYAryxJzmU",
    "client_name": "mimir-api",
    "grant_types": ["client_credentials"],
    "scopes": ["openid", "profile", "email"]
  }'
```

### ✅ วิธี D: ติดต่อ Asgard Admin

ส่ง credentials นี้ให้ Asgard admin เพื่อลงทะเบียน:
```
Client ID: 368876822349333070
Client Secret: CeGcfzog8DcJ4cxK658tRtCeEgFwdt1AAU60twz5Su1qIpsGtfdqtq8NYAryxJzmU
```

---

## ตัวอักษรที่ 3: ทดสอบ Token

หลังจากลงทะเบียนแล้ว ให้รัน:

```bash
# ได้ token
TOKEN=$(curl -s -X POST http://localhost:8085/oauth/v2/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials" \
  -d "client_id=368876822349333070" \
  -d "client_secret=CeGcfzog8DcJ4cxK658tRtCeEgFwdt1AAU60twz5Su1qIpsGtfdqtq8NYAryxJzmU" \
  -d "scope=openid" | jq -r '.access_token')

echo "Token: $TOKEN"

# ทดสอบว่าใช้ได้
curl -s -H "Authorization: Bearer $TOKEN" http://localhost:8080/health | jq .
```

---

## ขั้นตอนที่ 4: รัน Integration Test

```bash
cd /Users/mimir/Developer/refgraph-rs

# ตั้ง token
export MIMIR_TOKEN="$TOKEN"
export MIMIR_URL="http://localhost:8080"

# รัน test
bash ./scripts/may-18-integration-test.sh
```

---

## คาดหวัง

✅ Health Check: PASS  
✅ Sample Data: PASS  
✅ Ingestion: PASS (ถ้า token ใช้ได้)  
✅ Hit Rate: รันคำค้นหา 10 ข้อ  
✅ Pipeline: E2E flow  

**เป้าหมาย:** Hit Rate ≥ 75% = PASS ✅

