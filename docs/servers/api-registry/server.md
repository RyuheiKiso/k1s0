# system-api-registry-server 險ｭ險・

system tier 縺ｮ OpenAPI/Protobuf 繧ｹ繧ｭ繝ｼ繝樣寔荳ｭ邂｡逅・し繝ｼ繝舌・縲ゅせ繧ｭ繝ｼ繝槭・逋ｻ骭ｲ繝ｻ繝舌・繧ｸ繝ｧ繝ｳ邂｡逅・・遐ｴ螢顔噪螟画峩讀懷・繝ｻ蟾ｮ蛻・｡ｨ遉ｺ繧呈署萓帙☆繧九３ust 螳溯｣・・

## 讎りｦ・

| 讖溯・ | 隱ｬ譏・|
| --- | --- |
| 繧ｹ繧ｭ繝ｼ繝樒匳骭ｲ繝ｻ繝舌・繧ｸ繝ｧ繝ｳ邂｡逅・| OpenAPI 3.x / Protobuf 繧ｹ繧ｭ繝ｼ繝槭・逋ｻ骭ｲ繝ｻ繝舌・繧ｸ繝ｧ繝ｳ螻･豁ｴ邂｡逅・|
| 繝舌Μ繝・・繧ｷ繝ｧ繝ｳ | OpenAPI Validator / buf lint 縺ｫ繧医ｋ逋ｻ骭ｲ譎ゅせ繧ｭ繝ｼ繝樊､懆ｨｼ |
| 遐ｴ螢顔噪螟画峩讀懷・ | 繝輔ぅ繝ｼ繝ｫ繝牙炎髯､繝ｻ蝙句､画峩繝ｻ蠢・亥喧遲峨・蠕梧婿莠呈鋤諤ｧ遐ｴ螢翫ｒ閾ｪ蜍墓､懷・ |
| 蟾ｮ蛻・｡ｨ遉ｺ | 繝舌・繧ｸ繝ｧ繝ｳ髢薙・繧ｹ繧ｭ繝ｼ繝槫ｷｮ蛻・ｒ讒矩蛹・JSON 縺ｧ蜿門ｾ・|
| 繧ｹ繧ｭ繝ｼ繝樊峩譁ｰ騾夂衍 | 繧ｹ繧ｭ繝ｼ繝樒匳骭ｲ繝ｻ譖ｴ譁ｰ譎ゅ↓ Kafka `k1s0.system.apiregistry.schema_updated.v1` 繧堤匱陦・|

### 謚陦薙せ繧ｿ繝・け

> 蜈ｱ騾壽橿陦薙せ繧ｿ繝・け縺ｯ [繝・Φ繝励Ξ繝ｼ繝井ｻ墓ｧ・繧ｵ繝ｼ繝舌・.md](../../templates/server/繧ｵ繝ｼ繝舌・.md#蜈ｱ騾壽橿陦薙せ繧ｿ繝・け) 繧貞盾辣ｧ縲・

| 繧ｳ繝ｳ繝昴・繝阪Φ繝・| Rust |
| --- | --- |
| 繧ｹ繧ｭ繝ｼ繝樊､懆ｨｼ | openapi-spec-validator・・ubprocess 蜻ｼ縺ｳ蜃ｺ縺暦ｼ・ buf lint・・ubprocess 蜻ｼ縺ｳ蜃ｺ縺暦ｼ・|

### 驟咲ｽｮ繝代せ

驟咲ｽｮ: `regions/system/server/rust/api-registry/`・・Tier蛻･驟咲ｽｮ繝代せ蜿ら・](../../templates/server/繧ｵ繝ｼ繝舌・.md#tier-蛻･驟咲ｽｮ繝代せ)・・

---

## API 螳夂ｾｩ

### REST API 繧ｨ繝ｳ繝峨・繧､繝ｳ繝・

蜈ｨ繧ｨ繝ｳ繝峨・繧､繝ｳ繝医・ [API險ｭ險・md](../../architecture/api/API險ｭ險・md) D-007 縺ｮ邨ｱ荳繧ｨ繝ｩ繝ｼ繝ｬ繧ｹ繝昴Φ繧ｹ縺ｫ蠕薙≧縲ゅお繝ｩ繝ｼ繧ｳ繝ｼ繝峨・繝励Ξ繝輔ぅ繝・け繧ｹ縺ｯ `SYS_APIREG_` 縺ｨ縺吶ｋ縲・

| Method | Path | Description | 隱榊庄 |
| --- | --- | --- | --- |
| GET | `/api/v1/schemas` | 繧ｹ繧ｭ繝ｼ繝樔ｸ隕ｧ蜿門ｾ・| `sys_auditor` 莉･荳・|
| POST | `/api/v1/schemas` | 繧ｹ繧ｭ繝ｼ繝樒匳骭ｲ・亥・蝗槭ヰ繝ｼ繧ｸ繝ｧ繝ｳ・・| `sys_operator` 莉･荳・|
| GET | `/api/v1/schemas/:name` | 繧ｹ繧ｭ繝ｼ繝槫叙蠕暦ｼ域怙譁ｰ繝舌・繧ｸ繝ｧ繝ｳ・・| `sys_auditor` 莉･荳・|
| GET | `/api/v1/schemas/:name/versions` | 繝舌・繧ｸ繝ｧ繝ｳ荳隕ｧ蜿門ｾ・| `sys_auditor` 莉･荳・|
| GET | `/api/v1/schemas/:name/versions/:version` | 迚ｹ螳壹ヰ繝ｼ繧ｸ繝ｧ繝ｳ蜿門ｾ・| `sys_auditor` 莉･荳・|
| POST | `/api/v1/schemas/:name/versions` | 譁ｰ繝舌・繧ｸ繝ｧ繝ｳ逋ｻ骭ｲ | `sys_operator` 莉･荳・|
| DELETE | `/api/v1/schemas/:name/versions/:version` | 繝舌・繧ｸ繝ｧ繝ｳ蜑企勁 | `sys_admin` 縺ｮ縺ｿ |
| POST | `/api/v1/schemas/:name/compatibility` | 莠呈鋤諤ｧ繝√ぉ繝・け・育ｴ螢顔噪螟画峩讀懷・・・| `sys_operator` 莉･荳・|
| GET | `/api/v1/schemas/:name/diff` | 繝舌・繧ｸ繝ｧ繝ｳ髢灘ｷｮ蛻・叙蠕・| `sys_auditor` 莉･荳・|
| GET | `/healthz` | 繝倥Ν繧ｹ繝√ぉ繝・け | 荳崎ｦ・|
| GET | `/readyz` | 繝ｬ繝・ぅ繝阪せ繝√ぉ繝・け | 荳崎ｦ・|
| GET | `/metrics` | Prometheus 繝｡繝医Μ繧ｯ繧ｹ | 荳崎ｦ・|

#### GET /api/v1/schemas

逋ｻ骭ｲ貂医∩繧ｹ繧ｭ繝ｼ繝樔ｸ隕ｧ繧偵・繝ｼ繧ｸ繝阪・繧ｷ繝ｧ繝ｳ莉倥″縺ｧ蜿門ｾ励☆繧九・

**繧ｯ繧ｨ繝ｪ繝代Λ繝｡繝ｼ繧ｿ**

| 繝代Λ繝｡繝ｼ繧ｿ | 蝙・| 蠢・・| 繝・ヵ繧ｩ繝ｫ繝・| 隱ｬ譏・|
| --- | --- | --- | --- | --- |
| `schema_type` | string | No | - | 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･縺ｧ繝輔ぅ繝ｫ繧ｿ・・penapi/protobuf・・|
| `page` | int | No | 1 | 繝壹・繧ｸ逡ｪ蜿ｷ |
| `page_size` | int | No | 20 | 1 繝壹・繧ｸ縺ゅ◆繧翫・莉ｶ謨ｰ |

**繝ｬ繧ｹ繝昴Φ繧ｹ繝輔ぅ繝ｼ繝ｫ繝会ｼ・00 OK・・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `schemas[].name` | string | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `schemas[].description` | string | 繧ｹ繧ｭ繝ｼ繝槭・隱ｬ譏・|
| `schemas[].schema_type` | string | 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･・・openapi` / `protobuf`・・|
| `schemas[].latest_version` | int | 譛譁ｰ繝舌・繧ｸ繝ｧ繝ｳ逡ｪ蜿ｷ |
| `schemas[].version_count` | int | 逋ｻ骭ｲ繝舌・繧ｸ繝ｧ繝ｳ謨ｰ |
| `schemas[].created_at` | string | 蛻晏屓逋ｻ骭ｲ譌･譎・|
| `schemas[].updated_at` | string | 譛邨よ峩譁ｰ譌･譎・|
| `pagination` | object | 繝壹・繧ｸ繝阪・繧ｷ繝ｧ繝ｳ・・otal_count, page, page_size, has_next・・|

#### POST /api/v1/schemas

繧ｹ繧ｭ繝ｼ繝槭ｒ譁ｰ隕冗匳骭ｲ縺吶ｋ縲ょ・蝗槭ヰ繝ｼ繧ｸ繝ｧ繝ｳ・・ersion 1・峨′菴懈・縺輔ｌ繧九ら匳骭ｲ譎ゅ↓繝舌Μ繝・・繧ｷ繝ｧ繝ｳ繧貞ｮ溯｡後＠縲√お繝ｩ繝ｼ縺後≠繧句ｴ蜷医・ 422 繧定ｿ斐☆縲・

**繝ｪ繧ｯ繧ｨ繧ｹ繝医ヵ繧｣繝ｼ繝ｫ繝・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 蠢・・| 隱ｬ譏・|
| --- | --- | --- | --- |
| `name` | string | Yes | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `description` | string | Yes | 繧ｹ繧ｭ繝ｼ繝槭・隱ｬ譏・|
| `schema_type` | string | Yes | 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･・・openapi` / `protobuf`・・|
| `content` | string | Yes | 繧ｹ繧ｭ繝ｼ繝樊悽譁・ｼ・AML/JSON/proto・・|
| `registered_by` | string | No | 逋ｻ骭ｲ閠・Θ繝ｼ繧ｶ繝ｼ ID・育怐逡･譎ゅ・ `anonymous`・・|

**繝ｬ繧ｹ繝昴Φ繧ｹ繝輔ぅ繝ｼ繝ｫ繝会ｼ・01 Created・・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | string | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `version` | int | 繝舌・繧ｸ繝ｧ繝ｳ逡ｪ蜿ｷ・・・・|
| `schema_type` | string | 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･ |
| `content_hash` | string | 繧ｳ繝ｳ繝・Φ繝・・ SHA-256 繝上ャ繧ｷ繝･ |
| `created_at` | string | 逋ｻ骭ｲ譌･譎・|

**繧ｨ繝ｩ繝ｼ繝ｬ繧ｹ繝昴Φ繧ｹ・・22・・*: `SYS_APIREG_SCHEMA_INVALID`・・etails 縺ｫ繝舌Μ繝・・繧ｷ繝ｧ繝ｳ繧ｨ繝ｩ繝ｼ荳隕ｧ・・

#### GET /api/v1/schemas/:name

謖・ｮ壹せ繧ｭ繝ｼ繝槭・譛譁ｰ繝舌・繧ｸ繝ｧ繝ｳ縺ｮ繧ｳ繝ｳ繝・Φ繝・ｒ蜿門ｾ励☆繧九・

**繝ｬ繧ｹ繝昴Φ繧ｹ繝輔ぅ繝ｼ繝ｫ繝会ｼ・00 OK・・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | string | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `description` | string | 繧ｹ繧ｭ繝ｼ繝槭・隱ｬ譏・|
| `schema_type` | string | 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･ |
| `latest_version` | int | 譛譁ｰ繝舌・繧ｸ繝ｧ繝ｳ逡ｪ蜿ｷ |
| `version_count` | int | 逋ｻ骭ｲ繝舌・繧ｸ繝ｧ繝ｳ謨ｰ |
| `latest_content` | string | 譛譁ｰ繝舌・繧ｸ繝ｧ繝ｳ縺ｮ繧ｹ繧ｭ繝ｼ繝樊悽譁・|
| `content_hash` | string | 繧ｳ繝ｳ繝・Φ繝・・ SHA-256 繝上ャ繧ｷ繝･ |
| `created_at` | string | 蛻晏屓逋ｻ骭ｲ譌･譎・|
| `updated_at` | string | 譛邨よ峩譁ｰ譌･譎・|

> `latest_content` 縺ｯ縲梧怙譁ｰ繝舌・繧ｸ繝ｧ繝ｳ蜿門ｾ・API・・ET /api/v1/schemas/:name・峨榊ｰら畑繝輔ぅ繝ｼ繝ｫ繝牙錐縺ｨ縺励※蝗ｺ螳壹☆繧九・ 
> `content` 縺ｯ縲檎音螳壹ヰ繝ｼ繧ｸ繝ｧ繝ｳ蜿門ｾ・API・・ET /api/v1/schemas/:name/versions/:version・峨阪〒縺ｮ縺ｿ菴ｿ逕ｨ縺吶ｋ縲・
**繧ｨ繝ｩ繝ｼ繝ｬ繧ｹ繝昴Φ繧ｹ・・04・・*: `SYS_APIREG_SCHEMA_NOT_FOUND`

#### GET /api/v1/schemas/:name/versions

謖・ｮ壹せ繧ｭ繝ｼ繝槭・蜈ｨ繝舌・繧ｸ繝ｧ繝ｳ荳隕ｧ繧偵・繝ｼ繧ｸ繝阪・繧ｷ繝ｧ繝ｳ莉倥″縺ｧ蜿門ｾ励☆繧九・

**繝ｬ繧ｹ繝昴Φ繧ｹ繝輔ぅ繝ｼ繝ｫ繝会ｼ・00 OK・・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | string | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `versions[].version` | int | 繝舌・繧ｸ繝ｧ繝ｳ逡ｪ蜿ｷ |
| `versions[].content_hash` | string | 繧ｳ繝ｳ繝・Φ繝・ワ繝・す繝･ |
| `versions[].breaking_changes` | bool | 遐ｴ螢顔噪螟画峩繝輔Λ繧ｰ |
| `versions[].registered_by` | string | 逋ｻ骭ｲ閠・Θ繝ｼ繧ｶ繝ｼ ID |
| `versions[].created_at` | string | 逋ｻ骭ｲ譌･譎・|
| `pagination` | object | 繝壹・繧ｸ繝阪・繧ｷ繝ｧ繝ｳ |

#### GET /api/v1/schemas/:name/versions/:version

謖・ｮ壹ヰ繝ｼ繧ｸ繝ｧ繝ｳ縺ｮ繧ｹ繧ｭ繝ｼ繝槭さ繝ｳ繝・Φ繝・ｒ蜿門ｾ励☆繧九・

**繝ｬ繧ｹ繝昴Φ繧ｹ繝輔ぅ繝ｼ繝ｫ繝会ｼ・00 OK・・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | string | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `version` | int | 繝舌・繧ｸ繝ｧ繝ｳ逡ｪ蜿ｷ |
| `schema_type` | string | 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･ |
| `content` | string | 繧ｹ繧ｭ繝ｼ繝樊悽譁・|
| `content_hash` | string | 繧ｳ繝ｳ繝・Φ繝・ワ繝・す繝･ |
| `breaking_changes` | bool | 遐ｴ螢顔噪螟画峩繝輔Λ繧ｰ |
| `registered_by` | string | 逋ｻ骭ｲ閠・|
| `created_at` | string | 逋ｻ骭ｲ譌･譎・|

**繧ｨ繝ｩ繝ｼ繝ｬ繧ｹ繝昴Φ繧ｹ・・04・・*: `SYS_APIREG_VERSION_NOT_FOUND`

#### POST /api/v1/schemas/:name/versions

譌｢蟄倥せ繧ｭ繝ｼ繝槭↓譁ｰ繝舌・繧ｸ繝ｧ繝ｳ繧堤匳骭ｲ縺吶ｋ縲ら匳骭ｲ蜑阪↓莠呈鋤諤ｧ繝√ぉ繝・け繧定・蜍募ｮ溯｡後＠縲∫ｴ螢顔噪螟画峩縺梧､懷・縺輔ｌ縺溷ｴ蜷医・繝輔Λ繧ｰ繧堤ｫ九※繧具ｼ育匳骭ｲ縺ｯ縺昴・縺ｾ縺ｾ陦後≧・峨・

**繝ｪ繧ｯ繧ｨ繧ｹ繝医ヵ繧｣繝ｼ繝ｫ繝・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 蠢・・| 隱ｬ譏・|
| --- | --- | --- | --- |
| `content` | string | Yes | 繧ｹ繧ｭ繝ｼ繝樊悽譁・|
| `registered_by` | string | No | 逋ｻ骭ｲ閠・Θ繝ｼ繧ｶ繝ｼ ID・育怐逡･譎ゅ・ `anonymous`・・|

**繝ｬ繧ｹ繝昴Φ繧ｹ繝輔ぅ繝ｼ繝ｫ繝会ｼ・01 Created・・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | string | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `version` | int | 繝舌・繧ｸ繝ｧ繝ｳ逡ｪ蜿ｷ |
| `content_hash` | string | 繧ｳ繝ｳ繝・Φ繝・ワ繝・す繝･ |
| `breaking_changes` | bool | 遐ｴ螢顔噪螟画峩繝輔Λ繧ｰ |
| `created_at` | string | 逋ｻ骭ｲ譌･譎・|

**繧ｨ繝ｩ繝ｼ繝ｬ繧ｹ繝昴Φ繧ｹ・・22・・*: `SYS_APIREG_SCHEMA_INVALID`

#### DELETE /api/v1/schemas/:name/versions/:version

謖・ｮ壹ヰ繝ｼ繧ｸ繝ｧ繝ｳ繧貞炎髯､縺吶ｋ縲よ怙譁ｰ繝舌・繧ｸ繝ｧ繝ｳ・医ヰ繝ｼ繧ｸ繝ｧ繝ｳ謨ｰ = 1・峨・蜑企勁縺ｧ縺阪↑縺・・

**繝ｬ繧ｹ繝昴Φ繧ｹ**: 204 No Content

**繧ｨ繝ｩ繝ｼ繝ｬ繧ｹ繝昴Φ繧ｹ・・09・・*: `SYS_APIREG_CANNOT_DELETE_LATEST`

#### POST /api/v1/schemas/:name/compatibility

謖・ｮ壹せ繧ｭ繝ｼ繝槭↓蟇ｾ縺励※蜈･蜉帙さ繝ｳ繝・Φ繝・・莠呈鋤諤ｧ繝√ぉ繝・け縺ｮ縺ｿ繧貞ｮ溯｡後☆繧具ｼ育匳骭ｲ縺励↑縺・ｼ峨・

**繝ｪ繧ｯ繧ｨ繧ｹ繝医ヵ繧｣繝ｼ繝ｫ繝・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 蠢・・| 隱ｬ譏・|
| --- | --- | --- | --- |
| `content` | string | Yes | 讀懆ｨｼ蟇ｾ雎｡縺ｮ繧ｹ繧ｭ繝ｼ繝樊悽譁・|
| `base_version` | int | No | 豈碑ｼ・ｯｾ雎｡繝舌・繧ｸ繝ｧ繝ｳ・育怐逡･譎ゅ・譛譁ｰ・・|

**繝ｬ繧ｹ繝昴Φ繧ｹ繝輔ぅ繝ｼ繝ｫ繝会ｼ・00 OK・・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | string | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `base_version` | int | 豈碑ｼ・ｯｾ雎｡繝舌・繧ｸ繝ｧ繝ｳ |
| `compatible` | bool | 蠕梧婿莠呈鋤諤ｧ繝輔Λ繧ｰ |
| `breaking_changes` | BreakingChange[] | 遐ｴ螢顔噪螟画峩縺ｮ繝ｪ繧ｹ繝・|
| `non_breaking_changes` | ChangeDetail[] | 髱樒ｴ螢顔噪螟画峩縺ｮ繝ｪ繧ｹ繝・|

#### GET /api/v1/schemas/:name/diff

2 縺､縺ｮ繝舌・繧ｸ繝ｧ繝ｳ髢薙・蟾ｮ蛻・ｒ蜿門ｾ励☆繧九・

**繧ｯ繧ｨ繝ｪ繝代Λ繝｡繝ｼ繧ｿ**

| 繝代Λ繝｡繝ｼ繧ｿ | 蝙・| 蠢・・| 繝・ヵ繧ｩ繝ｫ繝・| 隱ｬ譏・|
| --- | --- | --- | --- | --- |
| `from` | int | No | `latest - 1` | 豈碑ｼ・・繝舌・繧ｸ繝ｧ繝ｳ |
| `to` | int | No | `latest` | 豈碑ｼ・・繝舌・繧ｸ繝ｧ繝ｳ |

**繝ｬ繧ｹ繝昴Φ繧ｹ繝輔ぅ繝ｼ繝ｫ繝会ｼ・00 OK・・*

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | string | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `from_version` | int | 豈碑ｼ・・繝舌・繧ｸ繝ｧ繝ｳ |
| `to_version` | int | 豈碑ｼ・・繝舌・繧ｸ繝ｧ繝ｳ |
| `breaking_changes` | bool | 遐ｴ螢顔噪螟画峩繝輔Λ繧ｰ |
| `diff.added` | ChangeDetail[] | 霑ｽ蜉縺輔ｌ縺溯ｦ∫ｴ |
| `diff.modified` | ChangeDetail[] | 螟画峩縺輔ｌ縺溯ｦ∫ｴ |
| `diff.removed` | ChangeDetail[] | 蜑企勁縺輔ｌ縺溯ｦ∫ｴ |

**繧ｨ繝ｩ繝ｼ繝ｬ繧ｹ繝昴Φ繧ｹ・・00・・*: `SYS_APIREG_VALIDATION_ERROR`

### 繧ｨ繝ｩ繝ｼ繧ｳ繝ｼ繝・

| 繧ｳ繝ｼ繝・| HTTP Status | 隱ｬ譏・|
| --- | --- | --- |
| `SYS_APIREG_SCHEMA_NOT_FOUND` | 404 | 謖・ｮ壹＆繧後◆繧ｹ繧ｭ繝ｼ繝槭′隕九▽縺九ｉ縺ｪ縺・|
| `SYS_APIREG_VERSION_NOT_FOUND` | 404 | 謖・ｮ壹＆繧後◆繝舌・繧ｸ繝ｧ繝ｳ縺瑚ｦ九▽縺九ｉ縺ｪ縺・|
| `SYS_APIREG_ALREADY_EXISTS` | 409 | 蜷御ｸ蜷阪・繧ｹ繧ｭ繝ｼ繝槭′譌｢縺ｫ蟄伜惠縺吶ｋ |
| `SYS_APIREG_CANNOT_DELETE_LATEST` | 409 | 蜚ｯ荳縺ｮ谿句ｭ倥ヰ繝ｼ繧ｸ繝ｧ繝ｳ縺ｯ蜑企勁縺ｧ縺阪↑縺・|
| `SYS_APIREG_SCHEMA_INVALID` | 422 | 繧ｹ繧ｭ繝ｼ繝槭・繝舌Μ繝・・繧ｷ繝ｧ繝ｳ繧ｨ繝ｩ繝ｼ・・penapi-spec-validator / buf lint・・|
| `SYS_APIREG_VALIDATION_ERROR` | 400 | 繝ｪ繧ｯ繧ｨ繧ｹ繝医ヱ繝ｩ繝｡繝ｼ繧ｿ縺ｮ繝舌Μ繝・・繧ｷ繝ｧ繝ｳ繧ｨ繝ｩ繝ｼ |
| `SYS_APIREG_VALIDATOR_ERROR` | 502 | 螟夜Κ繝舌Μ繝・・繧ｿ繝ｼ・・penapi-spec-validator / buf・峨・螳溯｡後お繝ｩ繝ｼ |
| `SYS_APIREG_INTERNAL_ERROR` | 500 | 蜀・Κ繧ｨ繝ｩ繝ｼ |

### gRPC 繧ｵ繝ｼ繝薙せ螳夂ｾｩ

proto 繝輔ぃ繧､繝ｫ縺ｯ `api/proto/k1s0/system/apiregistry/v1/api_registry.proto` 縺ｫ驟咲ｽｮ縺吶ｋ縲・

```protobuf
syntax = "proto3";
package k1s0.system.apiregistry.v1;

service ApiRegistryService {
  rpc GetSchema(GetSchemaRequest) returns (GetSchemaResponse);
  rpc GetSchemaVersion(GetSchemaVersionRequest) returns (GetSchemaVersionResponse);
  rpc CheckCompatibility(CheckCompatibilityRequest) returns (CheckCompatibilityResponse);
}

message GetSchemaRequest {
  string name = 1;
}

message GetSchemaResponse {
  ApiSchemaProto schema = 1;
  string latest_content = 2;
}

message GetSchemaVersionRequest {
  string name = 1;
  uint32 version = 2;
}

message GetSchemaVersionResponse {
  ApiSchemaVersionProto version = 1;
}

message CheckCompatibilityRequest {
  string name = 1;
  string content = 2;
  optional uint32 base_version = 3;
}

message CheckCompatibilityResponse {
  string name = 1;
  uint32 base_version = 2;
  CompatibilityResultProto result = 3;
}

message ApiSchemaProto {
  string name = 1;
  string description = 2;
  string schema_type = 3;
  uint32 latest_version = 4;
  uint32 version_count = 5;
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp updated_at = 7;
}

message ApiSchemaVersionProto {
  string name = 1;
  uint32 version = 2;
  string schema_type = 3;
  string content = 4;
  string content_hash = 5;
  bool breaking_changes = 6;
  string registered_by = 7;
  google.protobuf.Timestamp created_at = 8;
}

message CompatibilityResultProto {
  bool compatible = 1;
  repeated ChangeDetail breaking_changes = 2;
  repeated ChangeDetail non_breaking_changes = 3;
}

message ChangeDetail {
  string change_type = 1;
  string path = 2;
  string description = 3;
}
```

---

## Kafka 繝｡繝・そ繝ｼ繧ｸ繝ｳ繧ｰ險ｭ險・

### 繧ｹ繧ｭ繝ｼ繝樊峩譁ｰ騾夂衍

繧ｹ繧ｭ繝ｼ繝槭・譁ｰ隕冗匳骭ｲ繝ｻ譁ｰ繝舌・繧ｸ繝ｧ繝ｳ逋ｻ骭ｲ繝ｻ繝舌・繧ｸ繝ｧ繝ｳ蜑企勁譎ゅ↓ Kafka 繝医ヴ繝・け `k1s0.system.apiregistry.schema_updated.v1` 縺ｫ繝｡繝・そ繝ｼ繧ｸ繧帝∽ｿ｡縺吶ｋ縲・

| 險ｭ螳夐・岼 | 蛟､ |
| --- | --- |
| 繝医ヴ繝・け | `k1s0.system.apiregistry.schema_updated.v1` |
| acks | `all` |
| message.timeout.ms | `5000` |
| 繧ｭ繝ｼ | 繧ｹ繧ｭ繝ｼ繝槫錐・井ｾ・ `k1s0-tenant-api`・・|

**繧､繝吶Φ繝育ｨｮ蛻･**:
- `SCHEMA_VERSION_REGISTERED` -- 譁ｰ繝舌・繧ｸ繝ｧ繝ｳ逋ｻ骭ｲ
- `SCHEMA_VERSION_DELETED` -- 繝舌・繧ｸ繝ｧ繝ｳ蜑企勁

---

## 繧｢繝ｼ繧ｭ繝・け繝√Ε

### 繧ｯ繝ｪ繝ｼ繝ｳ繧｢繝ｼ繧ｭ繝・け繝√Ε 繝ｬ繧､繝､繝ｼ

[繝・Φ繝励Ξ繝ｼ繝井ｻ墓ｧ・繧ｵ繝ｼ繝舌・.md](../../templates/server/繧ｵ繝ｼ繝舌・.md) 縺ｮ 4 繝ｬ繧､繝､繝ｼ讒区・縺ｫ蠕薙≧縲・

| 繝ｬ繧､繝､繝ｼ | 繝｢繧ｸ繝･繝ｼ繝ｫ | 雋ｬ蜍・|
| --- | --- | --- |
| domain/entity | `ApiSchema`, `ApiSchemaVersion`, `CompatibilityResult`, `SchemaDiff` | 繧ｨ繝ｳ繝・ぅ繝・ぅ螳夂ｾｩ |
| domain/repository | `ApiSchemaRepository`, `ApiSchemaVersionRepository` | 繝ｪ繝昴ず繝医Μ繝医Ξ繧､繝・|
| domain/service | `ApiRegistryDomainService` | 遐ｴ螢顔噪螟画峩讀懷・繝ｭ繧ｸ繝・け繝ｻ蟾ｮ蛻・ｮ怜・繝ｻ繧ｳ繝ｳ繝・Φ繝・ワ繝・す繝･險育ｮ・|
| usecase | `ListSchemasUsecase`, `RegisterSchemaUsecase`, `GetSchemaUsecase`, `ListVersionsUsecase`, `GetSchemaVersionUsecase`, `RegisterVersionUsecase`, `DeleteVersionUsecase`, `CheckCompatibilityUsecase`, `GetDiffUsecase` | 繝ｦ繝ｼ繧ｹ繧ｱ繝ｼ繧ｹ |
| adapter/handler | REST 繝上Φ繝峨Λ繝ｼ・・xum・・ gRPC 繝上Φ繝峨Λ繝ｼ・・onic・・| 繝励Ο繝医さ繝ｫ螟画鋤 |
| infrastructure/config | Config 繝ｭ繝ｼ繝繝ｼ | config.yaml 縺ｮ隱ｭ縺ｿ霎ｼ縺ｿ |
| infrastructure/persistence | `ApiSchemaPostgresRepository`, `ApiSchemaVersionPostgresRepository` | PostgreSQL 繝ｪ繝昴ず繝医Μ螳溯｣・|
| infrastructure/validator | `OpenApiValidator`, `ProtobufValidator` | subprocess 邨檎罰繝舌Μ繝・・繧ｿ繝ｼ螳溯｣・|
| infrastructure/messaging | `SchemaUpdatedKafkaProducer` | Kafka 繝励Ο繝・Η繝ｼ繧ｵ繝ｼ・医せ繧ｭ繝ｼ繝樊峩譁ｰ騾夂衍・・|

### 繝峨Γ繧､繝ｳ繝｢繝・Ν

#### ApiSchema

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | String | 繧ｹ繧ｭ繝ｼ繝槫錐・井ｾ・ `k1s0-tenant-api`・・|
| `description` | String | 繧ｹ繧ｭ繝ｼ繝槭・隱ｬ譏・|
| `schema_type` | String | 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･・・openapi` / `protobuf`・・|
| `latest_version` | u32 | 譛譁ｰ繝舌・繧ｸ繝ｧ繝ｳ逡ｪ蜿ｷ |
| `version_count` | u32 | 逋ｻ骭ｲ繝舌・繧ｸ繝ｧ繝ｳ謨ｰ |
| `created_at` | DateTime\<Utc\> | 蛻晏屓逋ｻ骭ｲ譌･譎・|
| `updated_at` | DateTime\<Utc\> | 譛邨よ峩譁ｰ譌･譎・|

#### ApiSchemaVersion

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `name` | String | 繧ｹ繧ｭ繝ｼ繝槫錐 |
| `version` | u32 | 繝舌・繧ｸ繝ｧ繝ｳ逡ｪ蜿ｷ・・ 蟋九∪繧翫・騾｣逡ｪ・・|
| `schema_type` | String | 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･ |
| `content` | String | 繧ｹ繧ｭ繝ｼ繝樊悽譁・ｼ・AML/JSON/proto・・|
| `content_hash` | String | 繧ｳ繝ｳ繝・Φ繝・・ SHA-256 繝上ャ繧ｷ繝･・磯㍾隍・､懷・縺ｫ菴ｿ逕ｨ・・|
| `breaking_changes` | bool | 蜑阪ヰ繝ｼ繧ｸ繝ｧ繝ｳ縺九ｉ縺ｮ遐ｴ螢顔噪螟画峩繝輔Λ繧ｰ |
| `breaking_change_details` | Vec\<BreakingChange\> | 遐ｴ螢顔噪螟画峩縺ｮ隧ｳ邏ｰ繝ｪ繧ｹ繝・|
| `registered_by` | String | 逋ｻ骭ｲ閠・・繝ｦ繝ｼ繧ｶ繝ｼ ID |
| `created_at` | DateTime\<Utc\> | 逋ｻ骭ｲ譌･譎・|

#### CompatibilityResult

| 繝輔ぅ繝ｼ繝ｫ繝・| 蝙・| 隱ｬ譏・|
| --- | --- | --- |
| `compatible` | bool | 蠕梧婿莠呈鋤諤ｧ繝輔Λ繧ｰ・育ｴ螢顔噪螟画峩縺ｪ縺・= true・・|
| `breaking_changes` | Vec\<BreakingChange\> | 遐ｴ螢顔噪螟画峩縺ｮ繝ｪ繧ｹ繝・|
| `non_breaking_changes` | Vec\<ChangeDetail\> | 髱樒ｴ螢顔噪螟画峩縺ｮ繝ｪ繧ｹ繝・|

#### BreakingChange・育ｴ螢顔噪螟画峩縺ｮ遞ｮ蛻･・・

| change_type | 隱ｬ譏・|
| --- | --- |
| `field_removed` | 繝ｬ繧ｹ繝昴Φ繧ｹ/繝ｪ繧ｯ繧ｨ繧ｹ繝医ヵ繧｣繝ｼ繝ｫ繝峨・蜑企勁 |
| `type_changed` | 繝輔ぅ繝ｼ繝ｫ繝峨・蝙句､画峩 |
| `required_added` | 繧ｪ繝励す繝ｧ繝ｳ繝輔ぅ繝ｼ繝ｫ繝峨・蠢・亥喧 |
| `path_removed` | API 繝代せ縺ｮ蜑企勁 |
| `method_removed` | HTTP繝｡繧ｽ繝・ラ縺ｮ蜑企勁・・ET/POST遲会ｼ・|
| `enum_value_removed` | enum 蛟､縺ｮ蜑企勁 |

---

## DB 繧ｹ繧ｭ繝ｼ繝・

PostgreSQL 縺ｮ `apiregistry` 繧ｹ繧ｭ繝ｼ繝槭↓莉･荳九・繝・・繝悶Ν繧帝・鄂ｮ縺吶ｋ縲・

```sql
CREATE SCHEMA IF NOT EXISTS apiregistry;

CREATE TABLE apiregistry.api_schemas (
    name         TEXT PRIMARY KEY,
    description  TEXT NOT NULL DEFAULT '',
    schema_type  TEXT NOT NULL CHECK (schema_type IN ('openapi', 'protobuf')),
    latest_version INTEGER NOT NULL DEFAULT 1,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE apiregistry.api_schema_versions (
    name                    TEXT NOT NULL REFERENCES apiregistry.api_schemas(name) ON DELETE CASCADE,
    version                 INTEGER NOT NULL,
    content                 TEXT NOT NULL,
    content_hash            TEXT NOT NULL,
    breaking_changes        BOOLEAN NOT NULL DEFAULT false,
    breaking_change_details JSONB NOT NULL DEFAULT '[]',
    registered_by           TEXT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (name, version)
);

CREATE INDEX idx_api_schema_versions_name ON apiregistry.api_schema_versions(name);
CREATE INDEX idx_api_schema_versions_content_hash ON apiregistry.api_schema_versions(content_hash);
```

---

## 險ｭ險域婿驥・

[隱崎ｨｼ隱榊庄險ｭ險・md](../../architecture/auth/隱崎ｨｼ隱榊庄險ｭ險・md) 縺ｮ RBAC 繝｢繝・Ν縺ｫ蝓ｺ縺･縺阪∽ｻ･荳九・譁ｹ驥昴〒螳溯｣・☆繧九・

| 鬆・岼 | 險ｭ險・|
| --- | --- |
| 螳溯｣・ｨ隱・| Rust |
| 繧ｹ繧ｭ繝ｼ繝樒ｨｮ蛻･ | `openapi`・・penAPI 3.x YAML/JSON・峨→ `protobuf`・・proto 繝輔ぃ繧､繝ｫ・峨・ 2 遞ｮ鬘槭ｒ繧ｵ繝昴・繝・|
| 繝舌Μ繝・・繧ｷ繝ｧ繝ｳ | 逋ｻ骭ｲ譎ゅ↓ subprocess 邨檎罰縺ｧ openapi-spec-validator・・penAPI・峨∪縺溘・ buf lint・・rotobuf・峨ｒ螳溯｡後＠讀懆ｨｼ繧ｨ繝ｩ繝ｼ繧定ｿ斐☆ |
| 遐ｴ螢顔噪螟画峩讀懷・ | 譁ｰ繝舌・繧ｸ繝ｧ繝ｳ逋ｻ骭ｲ譎ゅ↓蜑阪ヰ繝ｼ繧ｸ繝ｧ繝ｳ縺ｨ縺ｮ豈碑ｼ・ｒ陦後＞縲√ヵ繧｣繝ｼ繝ｫ繝牙炎髯､繝ｻ蝙句､画峩繝ｻ蠢・亥喧繝ｻ繝代せ蜑企勁遲峨・螟画峩繧呈､懷・縺吶ｋ |
| 蟾ｮ蛻・｡ｨ遉ｺ | 繝舌・繧ｸ繝ｧ繝ｳ髢薙・蟾ｮ蛻・ｒ `added` / `modified` / `removed` 縺ｫ蛻・｡槭＠縺滓ｧ矩蛹・JSON 縺ｧ謠蝉ｾ帙☆繧・|
| kafka-schemaregistry 縺ｨ縺ｮ蟇ｾ豈・| kafka-schemaregistry 繝ｩ繧､繝悶Λ繝ｪ縺ｯ Kafka Avro 繧ｹ繧ｭ繝ｼ繝槫髄縺代ょｽ薙し繝ｼ繝舌・縺ｯ REST/gRPC 繧ｹ繧ｭ繝ｼ繝槭・繝ｬ繧ｸ繧ｹ繝医Μ縺ｨ縺励※讖溯・縺吶ｋ |
| DB | PostgreSQL 縺ｮ `apiregistry` 繧ｹ繧ｭ繝ｼ繝橸ｼ・pi_schemas, api_schema_versions 繝・・繝悶Ν・・|
| Kafka | 繝励Ο繝・Η繝ｼ繧ｵ繝ｼ・・k1s0.system.apiregistry.schema_updated.v1`・・|
| 隱崎ｨｼ | JWT縺ｫ繧医ｋ隱榊庄縲らｮ｡逅・ｳｻ繧ｨ繝ｳ繝峨・繧､繝ｳ繝医・ `sys_operator` / `sys_admin` 繝ｭ繝ｼ繝ｫ縺悟ｿ・ｦ・|
| 繝昴・繝・| 8080・・EST・・ 50051・・RPC・・|

---

## API 繝ｪ繧ｯ繧ｨ繧ｹ繝医・繝ｬ繧ｹ繝昴Φ繧ｹ萓・

### GET /api/v1/schemas

```json
{
  "schemas": [
    {
      "name": "k1s0-tenant-api",
      "description": "繝・リ繝ｳ繝育ｮ｡逅・API 繧ｹ繧ｭ繝ｼ繝・,
      "schema_type": "openapi",
      "latest_version": 3,
      "version_count": 3,
      "created_at": "2026-02-10T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    },
    {
      "name": "k1s0-notification-proto",
      "description": "騾夂衍繧ｵ繝ｼ繝薙せ Protobuf 繧ｹ繧ｭ繝ｼ繝・,
      "schema_type": "protobuf",
      "latest_version": 1,
      "version_count": 1,
      "created_at": "2026-02-15T10:00:00.000+00:00",
      "updated_at": "2026-02-15T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 12,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

### POST /api/v1/schemas

**繝ｪ繧ｯ繧ｨ繧ｹ繝茨ｼ・penAPI・・*

```json
{
  "name": "k1s0-tenant-api",
  "description": "繝・リ繝ｳ繝育ｮ｡逅・API 繧ｹ繧ｭ繝ｼ繝・,
  "schema_type": "openapi",
  "content": "openapi: 3.0.3
info:
  title: Tenant API
  version: 1.0.0
paths:
  /api/v1/tenants:
    get:
      summary: 繝・リ繝ｳ繝井ｸ隕ｧ
      responses:
        '200':
          description: OK
"
}
```

**繝ｪ繧ｯ繧ｨ繧ｹ繝茨ｼ・rotobuf・・*

```json
{
  "name": "k1s0-notification-proto",
  "description": "騾夂衍繧ｵ繝ｼ繝薙せ Protobuf 繧ｹ繧ｭ繝ｼ繝・,
  "schema_type": "protobuf",
  "content": "syntax = \"proto3\";
package k1s0.system.notification.v1;

service NotificationService {
  rpc SendNotification(SendNotificationRequest) returns (SendNotificationResponse);
}

message SendNotificationRequest {
  string channel_id = 1;
  string recipient = 2;
}

message SendNotificationResponse {
  string notification_id = 1;
  string status = 2;
}
"
}
```

**繝ｬ繧ｹ繝昴Φ繧ｹ・・01 Created・・*

```json
{
  "name": "k1s0-tenant-api",
  "version": 1,
  "schema_type": "openapi",
  "content_hash": "sha256:a1b2c3d4e5f6...",
  "created_at": "2026-02-20T10:00:00.000+00:00"
}
```

**繝ｬ繧ｹ繝昴Φ繧ｹ・・22 Unprocessable Entity・・*

```json
{
  "error": {
    "code": "SYS_APIREG_SCHEMA_INVALID",
    "message": "schema validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "content", "message": "[line 5] info.version is required"},
      {"field": "content", "message": "[line 12] response schema must have a type"}
    ]
  }
}
```

### GET /api/v1/schemas/:name

```json
{
  "name": "k1s0-tenant-api",
  "description": "繝・リ繝ｳ繝育ｮ｡逅・API 繧ｹ繧ｭ繝ｼ繝・,
  "schema_type": "openapi",
  "latest_version": 3,
  "version_count": 3,
  "latest_content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 3.0.0\n...",
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "created_at": "2026-02-10T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
}
```

### GET /api/v1/schemas/:name/versions

```json
{
  "name": "k1s0-tenant-api",
  "versions": [
    {
      "version": 3,
      "content_hash": "sha256:f6e5d4c3b2a1...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-20T12:30:00.000+00:00"
    },
    {
      "version": 2,
      "content_hash": "sha256:e5d4c3b2a1f6...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-15T10:00:00.000+00:00"
    },
    {
      "version": 1,
      "content_hash": "sha256:a1b2c3d4e5f6...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-10T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 3,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

### GET /api/v1/schemas/:name/versions/:version

```json
{
  "name": "k1s0-tenant-api",
  "version": 2,
  "schema_type": "openapi",
  "content": "openapi: 3.0.3
info:
  title: Tenant API
  version: 2.0.0
...",
  "content_hash": "sha256:e5d4c3b2a1f6...",
  "breaking_changes": false,
  "registered_by": "user-001",
  "created_at": "2026-02-15T10:00:00.000+00:00"
}
```

### POST /api/v1/schemas/:name/versions

**繝ｪ繧ｯ繧ｨ繧ｹ繝・*

```json
{
  "content": "openapi: 3.0.3
info:
  title: Tenant API
  version: 3.0.0
paths:
  /api/v1/tenants:
    get:
      summary: 繝・リ繝ｳ繝井ｸ隕ｧ
      ...
",
  "registered_by": "user-001"
}
```

**繝ｬ繧ｹ繝昴Φ繧ｹ・・01 Created・・*

```json
{
  "name": "k1s0-tenant-api",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": false,
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

**繝ｬ繧ｹ繝昴Φ繧ｹ・・01 Created -- 遐ｴ螢顔噪螟画峩縺ゅｊ・・*

```json
{
  "name": "k1s0-tenant-api",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": true,
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

### DELETE /api/v1/schemas/:name/versions/:version

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_APIREG_CANNOT_DELETE_LATEST",
    "message": "cannot delete the only remaining version of schema: k1s0-tenant-api",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/schemas/:name/compatibility

**繝ｪ繧ｯ繧ｨ繧ｹ繝・*

```json
{
  "content": "openapi: 3.0.3
info:
  title: Tenant API
  version: 4.0.0
...",
  "base_version": 3
}
```

**繝ｬ繧ｹ繝昴Φ繧ｹ・・00 OK・・*

```json
{
  "name": "k1s0-tenant-api",
  "base_version": 3,
  "compatible": false,
  "breaking_changes": [
    {
      "change_type": "field_removed",
      "path": "/api/v1/tenants GET response.properties.name",
      "description": "繝輔ぅ繝ｼ繝ｫ繝・'name' 縺悟炎髯､縺輔ｌ縺ｾ縺励◆"
    }
  ],
  "non_breaking_changes": [
    {
      "change_type": "field_added",
      "path": "/api/v1/tenants GET response.properties.display_name",
      "description": "繝輔ぅ繝ｼ繝ｫ繝・'display_name' 縺瑚ｿｽ蜉縺輔ｌ縺ｾ縺励◆"
    }
  ]
}
```

### GET /api/v1/schemas/:name/diff

**繝ｬ繧ｹ繝昴Φ繧ｹ・・00 OK・・*

```json
{
  "name": "k1s0-tenant-api",
  "from_version": 2,
  "to_version": 3,
  "breaking_changes": false,
  "diff": {
    "added": [
      {
        "path": "/api/v1/tenants GET response.properties.display_name",
        "type": "object",
        "description": "譁ｰ繝輔ぅ繝ｼ繝ｫ繝・ display_name・郁｡ｨ遉ｺ蜷搾ｼ・
      }
    ],
    "modified": [
      {
        "path": "/api/v1/tenants GET summary",
        "before": "繝・リ繝ｳ繝井ｸ隕ｧ",
        "after": "繝・リ繝ｳ繝井ｸ隕ｧ蜿門ｾ・
      }
    ],
    "removed": []
  }
}
```

---

## Kafka 繝｡繝・そ繝ｼ繧ｸ繝輔か繝ｼ繝槭ャ繝・

### 譁ｰ繝舌・繧ｸ繝ｧ繝ｳ逋ｻ骭ｲ

```json
{
  "event_type": "SCHEMA_VERSION_REGISTERED",
  "schema_name": "k1s0-tenant-api",
  "schema_type": "openapi",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": false,
  "registered_by": "user-001",
  "timestamp": "2026-02-20T12:30:00.000+00:00"
}
```

### 繝舌・繧ｸ繝ｧ繝ｳ蜑企勁

```json
{
  "event_type": "SCHEMA_VERSION_DELETED",
  "schema_name": "k1s0-tenant-api",
  "schema_type": "openapi",
  "version": 1,
  "deleted_by": "user-001",
  "timestamp": "2026-02-20T15:00:00.000+00:00"
}
```

---

## 萓晏ｭ倬未菫ょ峙

```
                    笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・
                    笏・                   adapter 螻､                    笏・
                    笏・ 笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・  笏・
                    笏・ 笏・REST Handler (apiregistry_handler.rs)    笏・  笏・
                    笏・ 笏・ healthz / readyz / metrics              笏・  笏・
                    笏・ 笏・ list_schemas / register_schema          笏・  笏・
                    笏・ 笏・ get_schema / list_versions              笏・  笏・
                    笏・ 笏・ get_schema_version                      笏・  笏・
                    笏・ 笏・ register_version / delete_version       笏・  笏・
                    笏・ 笏・ check_compatibility / get_diff          笏・  笏・
                    笏・ 笏懌楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏､   笏・
                    笏・ 笏・gRPC Handler (apiregistry_grpc.rs)       笏・  笏・
                    笏・ 笏・ GetSchema / GetSchemaVersion            笏・  笏・
                    笏・ 笏・ CheckCompatibility                      笏・  笏・
                    笏・ 笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏ｬ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・  笏・
                    笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏ｼ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・
                                              笏・
                    笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笆ｼ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・
                    笏・                  usecase 螻､                    笏・
                    笏・ ListSchemasUsecase / RegisterSchemaUsecase /   笏・
                    笏・ GetSchemaUsecase / ListVersionsUsecase /       笏・
                    笏・ GetSchemaVersionUsecase /                      笏・
                    笏・ RegisterVersionUsecase / DeleteVersionUsecase /笏・
                    笏・ CheckCompatibilityUsecase / GetDiffUsecase     笏・
                    笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏ｬ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・
                                              笏・
              笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏ｼ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・
              笏・                              笏・                      笏・
    笏娯楳笏笏笏笏笏笏笏笏笆ｼ笏笏笏笏笏笏笏・             笏娯楳笏笏笏笏笏笏笏笏笆ｼ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・  笏・
    笏・ domain/entity  笏・             笏・domain/repository          笏・  笏・
    笏・ ApiSchema,     笏・             笏・ApiSchemaRepository        笏・  笏・
    笏・ ApiSchemaVer,  笏・             笏・ApiSchemaVersionRepository 笏・  笏・
    笏・ Compatibility  笏・             笏・(trait)                    笏・  笏・
    笏・ Result,        笏・             笏披楳笏笏笏笏笏笏笏笏笏笏ｬ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・  笏・
    笏・ SchemaDiff     笏・                        笏・                    笏・
    笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・                        笏・                    笏・
              笏・                               笏・                    笏・
              笏・ 笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・           笏・                    笏・
              笏披楳笏笆ｶ domain/service 笏・           笏・                    笏・
                 笏・ApiRegistry    笏・           笏・                    笏・
                 笏・DomainService  笏・           笏・                    笏・
                 笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・           笏・                    笏・
                    笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏ｼ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・
                    笏・            infrastructure 螻､  笏・
                    笏・ 笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏・ 笏娯楳笏笏笏笏笆ｼ笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・ 笏・
                    笏・ 笏・Kafka        笏・ 笏・ApiSchemaPostgres       笏・ 笏・
                    笏・ 笏・Producer     笏・ 笏・Repository             笏・ 笏・
                    笏・ 笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏・ 笏懌楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏､  笏・
                    笏・ 笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏・ 笏・ApiSchemaVersion       笏・ 笏・
                    笏・ 笏・OpenApi      笏・ 笏・PostgresRepository     笏・ 笏・
                    笏・ 笏・Validator    笏・ 笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・ 笏・
                    笏・ 笏懌楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏､  笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・ 笏・
                    笏・ 笏・Protobuf     笏・ 笏・Database               笏・ 笏・
                    笏・ 笏・Validator    笏・ 笏・Config                 笏・ 笏・
                    笏・ 笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏・ 笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・ 笏・
                    笏・ 笏娯楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏・                             笏・
                    笏・ 笏・Config       笏・                             笏・
                    笏・ 笏・Loader       笏・                             笏・
                    笏・ 笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏・                             笏・
                    笏披楳笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏笏・
```

---

## 險ｭ螳壹ヵ繧｡繧､繝ｫ萓・

### config.yaml・域悽逡ｪ・・

```yaml
app:
  name: "api-registry"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 50051

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic: "k1s0.system.apiregistry.schema_updated.v1"

validator:
  openapi_spec_validator_path: "/usr/local/bin/openapi-spec-validator"
  buf_path: "/usr/local/bin/buf"
  timeout_seconds: 30
```

---

## 繝・・繝ｭ繧､

### Helm values

[helm險ｭ險・md](../../infrastructure/kubernetes/helm險ｭ險・md) 縺ｮ繧ｵ繝ｼ繝舌・逕ｨ Helm Chart 繧剃ｽｿ逕ｨ縺吶ｋ縲Ｂpi-registry 蝗ｺ譛峨・ values 縺ｯ莉･荳九・騾壹ｊ縲・

```yaml
# values-api-registry.yaml・・nfra/helm/services/system/api-registry/values.yaml・・
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/api-registry
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 4
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/api-registry/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

### Vault 繧ｷ繝ｼ繧ｯ繝ｬ繝・ヨ繝代せ

| 繧ｷ繝ｼ繧ｯ繝ｬ繝・ヨ | 繝代せ |
| --- | --- |
| DB 繝代せ繝ｯ繝ｼ繝・| `secret/data/k1s0/system/api-registry/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

## 髢｢騾｣繝峨く繝･繝｡繝ｳ繝・

> 蜈ｱ騾夐未騾｣繝峨く繝･繝｡繝ｳ繝医・ [deploy.md](../_common/deploy.md#蜈ｱ騾夐未騾｣繝峨く繝･繝｡繝ｳ繝・ 繧貞盾辣ｧ縲・

- [system-server.md](../auth/server.md) -- system tier 繧ｵ繝ｼ繝舌・荳隕ｧ
- [system-library-schemaregistry.md](../../libraries/data/schemaregistry.md) -- Kafka Avro 繧ｹ繧ｭ繝ｼ繝槭Ξ繧ｸ繧ｹ繝医Μ繝ｩ繧､繝悶Λ繝ｪ・・afka 蜷代￠縲∝ｽ薙し繝ｼ繝舌・縺ｯ REST/gRPC 蜷代￠・・
- [proto險ｭ險・md](../../architecture/api/proto險ｭ險・md) -- Protobuf 繧ｹ繧ｭ繝ｼ繝櫁ｨｭ險医ぎ繧､繝峨Λ繧､繝ｳ
- [gRPC險ｭ險・md](../../architecture/api/gRPC險ｭ險・md) -- gRPC 險ｭ險医ぎ繧､繝峨Λ繧､繝ｳ



