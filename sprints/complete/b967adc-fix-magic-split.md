# fix: 마력 스탯 3분할 — 몽환의 결정 이중 반영 문제 수정

**커밋**: `b967adc`  
**브랜치**: `main`  
**날짜**: 2026-05-17

---

## 변경 동기

전투력 공식에는 `마력(기본 수치) × 마력%` 항이 포함됩니다. 그런데 `마력(기본 수치) = 마력(기본 수치, 기본) + 마력(기본 수치, 장비 아이템)`이며, `마력(기본 수치, 장비 아이템)`에는 현재 장착 중인 몽환의 결정에 의한 마력이 포함되어 있습니다.

이 프로그램은 몽환의 결정 조합 교체를 최적화하는 도구입니다. 기존에는 `stats.json`에 단일 `마력(기본)` 필드 하나만 저장했는데, 이 값이 이미 현재 장착 결정에 의한 마력을 포함하고 있었습니다. 따라서 최적화 계산 시 현재 장착 결정의 마력이 기저값에 한 번, 테스트 조합의 마력으로 또 한 번 — 이중으로 반영되는 문제가 있었습니다.

## 변경 내용

### 핵심 수정 (`src/stats.rs`)

- `magic: i32` 단일 필드를 세 필드로 분리:
  - `magic_base` ← `마력(기본 수치, 기본)`
  - `magic_equip` ← `마력(기본 수치, 장비 아이템)` (현재 장착 결정 포함)
  - `magic_crystal` ← `마력(몽환의 결정)` (현재 장착 결정에 의한 마력 합계)
- `effective_magic()` 메서드 추가: `magic_base + magic_equip - magic_crystal`
  - 이 값이 현재 장착 결정의 영향을 제거한 순수 기저 마력으로, 최적화 계산의 출발점이 됩니다.
- `get()` / `set()` 인덱스 5개 → 7개로 확장 (TUI 편집 자동 지원)
- `FIELD_NAMES` 배열 5개 → 7개로 확장

### 옵티마이저 수정 (`src/optimizer.rs`)

- `calculate_strength` 내 `let mut magic = base.magic` → `base.effective_magic()` 로 변경

### 기타

- `stats.json` 형식: 단일 `마력(기본)` → 세 필드(`마력(기본 수치, 기본)`, `마력(기본 수치, 장비 아이템)`, `마력(몽환의 결정)`) 로 변경
- `assets/D2CodingLigature.ttc` 폰트 파일 삭제 및 `.gitignore`에 `backends/`, `documents/` 추가
- `README.md` 아이템 → 몽환의 결정 표현 통일

## 계산 예시

`stats.json`에 `마력(기본 수치, 기본): 1000`, `마력(기본 수치, 장비 아이템): 500`, `마력(몽환의 결정): 100`이 저장된 경우:

- 기저 마력 = 1000 + 500 - 100 = **1400**
- 여기에 테스트 조합의 마력 수치를 더해 전투력 계산

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/stats.rs` | `magic` 필드 3분할, `effective_magic()` 추가, 인덱스 7개로 확장, 테스트 추가 |
| `src/optimizer.rs` | `base.magic` → `base.effective_magic()` |
| `stats.json` | 3-필드 형식으로 업데이트 |
| `README.md` | 아이템→몽환의 결정 표현 통일, macOS 실행 방법 추가 |
| `.gitignore` | `backends/`, `documents/`, `**/*.json.bak` 추가 |
| `assets/D2CodingLigature.ttc` | 삭제 |
