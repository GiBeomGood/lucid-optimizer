# Sprint 01 — TUI 아이템 관리 CLI

## 목표

JSON 파일에 저장된 아이템 정보를 터미널 UI로 조회·추가·수정·삭제할 수 있는 CLI 프로그램 구현. 변경 사항은 메모리에 먼저 반영되며, 명시적 저장 명령으로 JSON 파일에 기록된다.

## 범위

- 기능 2 (TUI 아이템 관리)만 다룸
- 기능 1 (최적 조합 탐색)은 별도 sprint

## 산출물

- `src/main.rs` — 엔트리 포인트, 터미널 셋업/티어다운, 메인 루프
- `src/item.rs` — 아이템·옵션 데이터 모델, serde 직렬화
- `src/storage.rs` — JSON 파일 로드/저장
- `src/app.rs` — 앱 상태(State), 모드(Mode) 관리, 액션 적용
- `src/ui.rs` — ratatui 렌더링 로직, 색상 테마
- `src/event.rs` — crossterm 키 이벤트 → 액션 매핑
- `items.json` — 기본 데이터 파일 (없으면 빈 배열로 생성)

## 데이터 모델

```rust
enum OptionKind {
    Magic,            // 마력
    MagicPercent,     // 마력%
    Mastery,          // 숙련도
    CritRate,         // 크리티컬 확률
    CritDamage,       // 크리티컬 데미지
    CooldownReduction // 재사용 대기시간 감소
}

struct ItemOption { kind: OptionKind, value: i32 }
struct Item { options: [ItemOption; 2] }
```

JSON 형식은 `documents/description.md` 명세 따름 (한글 키 사용).

## 색상 테마

Claude Code에서 파일 경로/스킬을 강조할 때 쓰는 연한 소라색~보라색(periwinkle) 톤을 강조 색으로 사용.

- `ACCENT` — `Color::Rgb(180, 170, 255)` 근처 (periwinkle). 선택 행, 편집 중 값 박스, 활성 힌트 키, 강조 라벨
- `ACCENT_DIM` — `Color::Rgb(120, 115, 180)` 근처. 비활성 강조(예: List에서 옵션 커서)
- `MUTED` — `Color::DarkGray`. 하단 힌트 텍스트
- `WARN` — `Color::Yellow`. dirty 표시, 삭제 1단 대기 표시
- `DANGER` — `Color::Red`. 삭제 확정 직전 강조

> 실제 RGB 값은 구현 시 시각적으로 가장 가까운 톤으로 미세 조정. 지원 안 되는 터미널은 ratatui가 자동으로 가까운 ANSI로 폴백.

## 앱 상태/모드

- `Mode::List` — 아이템 목록 탐색 (기본). **아이템 단위 위/아래 이동만 가능**
- `Mode::Edit { item_idx, option_idx }` — 특정 아이템 진입 상태. 좌우로 옵션1↔옵션2 이동, Enter로 값 입력, Esc로 List 복귀
- `Mode::EditValue { item_idx, option_idx, buffer }` — 값 인라인 수정 (Edit 안에서 Enter로 진입)
- `Mode::Adding(AddStep)` — 추가 흐름 (옵션1 선택 → 값1 입력 → 옵션2 선택 → 값2 입력 → 확정)
- `Mode::ConfirmDelete { item_idx }` — `d` 한 번 누른 상태. 화면에 명시적 표시. `d` 한 번 더로 삭제, `Esc`/다른 키로 취소
- `dirty: bool` — 저장되지 않은 변경 여부
- `flash: Option<(String, Instant)>` — 일시적 알림(저장 완료 등) 짧게 표시

### 모드 전이 근거

평소에는 보기 모드(List)로 가볍게 탐색하고, 편집 의도가 있을 때만 Edit으로 들어가서 옵션 간 이동·수정이 가능하게 한다. 의도하지 않은 값 변경을 막고, 키 바인딩이 컨텍스트별로 깔끔히 분리됨.

## 키 바인딩

| 컨텍스트 | 키 | 동작 |
|---|---|---|
| List | `↑`/`k`, `↓`/`j` | 아이템 간 이동 |
| List | `Enter` 또는 `→`/`l` | 선택 아이템의 Edit 모드 진입 (option_idx=0) |
| List | `a` | 아이템 추가 모드 진입 |
| List | `d` | 삭제 1단 (ConfirmDelete 진입) |
| List | `s` | JSON 파일에 저장 |
| List | `q` | 종료 (dirty면 확인) |
| Edit | `←`/`h`, `→`/`l` | 옵션1↔옵션2 커서 이동 |
| Edit | `Enter` | 현재 옵션 값 편집(EditValue) 진입 |
| Edit | `Esc` 또는 `↑`/`↓` | List 복귀 (위/아래는 복귀 + 이동) |
| EditValue | 숫자/`-`/`Backspace` | 버퍼 편집 |
| EditValue | `Enter` | 적용 후 Edit 복귀 |
| EditValue | `Esc` | 취소 후 Edit 복귀 |
| Adding | `↑`/`↓` | 옵션 종류 선택 / 값 입력 단계에선 숫자 |
| Adding | `Enter` | 다음 단계 / 마지막에서 확정 |
| Adding | `Esc` | 추가 취소 (전체 롤백) |
| ConfirmDelete | `d` | 삭제 확정 |
| ConfirmDelete | 그 외 | 취소 (List 복귀) |

## UI 레이아웃

```
┌─ Items (n)                                    ● 저장 안 됨 ┐
│   1. 마력: 10                                              │
│      크리티컬 확률: 7                                      │
│ ▶ 2. 크리티컬 데미지: 8        ← periwinkle                │
│      크리티컬 확률: 13                                     │
│   3. 마력%: 5                                              │
│      숙련도: 12                                            │
│   ...                                                      │
│   + 아이템 추가하기                                        │
└────────────────────────────────────────────────────────────┘
 ↑↓: 이동 | Enter: 편집 | a: 추가 | d: 삭제 | s: 저장 | q: 종료   (회색)
```

### 시각 규칙

- **선택 행**: 좌측 `▶` + periwinkle 배경 또는 텍스트
- **Edit 모드**: 해당 옵션 값에 periwinkle 언더라인/배경
- **EditValue 모드**: 값 자리에 `[123_]` (커서 포함) periwinkle 박스
- **Dirty 표시**: 우측 상단 타이틀 옆 `● 저장 안 됨` (WARN 색). 저장 완료 시 `✓ 저장됨` flash 1.5s
- **ConfirmDelete 표시**:
  - 해당 아이템 좌측 `▶`가 빨간색으로 변경
  - 하단 힌트가 `한 번 더 d: 삭제 확정 | Esc: 취소`로 전환되며 WARN/DANGER 색
  - 가능하면 해당 행에 `(삭제하려면 d 한 번 더)` 인라인 표시
- **빈 파일**: 중앙에 "아이템이 없습니다. `a`를 눌러 추가하세요." (`a` 강조)
- **하단 힌트**: 모드별 동적, 활성 키 글자만 periwinkle, 설명은 MUTED

## 추가 기능 제안

- **`u` Undo / `Ctrl+r` Redo** — 메모리 상태 스택. 잘못된 편집/삭제 복구. 저장과 무관.
- **종료 시 dirty 가드 모달** — `q` 누르면 `저장 안 된 변경이 있습니다. s: 저장 후 종료 / q: 그냥 종료 / Esc: 취소`
- **JSON 파일 경로 인자** — `cargo run -- path/to/items.json`. 미지정 시 `items.json` 기본값
- **자동 백업** — 저장 시 기존 파일을 `items.json.bak`으로 한 번 복사 (옵션, 안전망)
- **상태 표시줄** — 아이템 개수, 현재 모드, dirty 상태를 한 줄로

## 구현 순서

1. **데이터 모델 + JSON I/O** — serde rename으로 한글 키, load/save, 단위 테스트
2. **터미널 셋업 스켈레톤** — raw mode, alternate screen, panic hook, `q` 종료
3. **테마 모듈** — periwinkle 등 색 상수 정의, `Style` 헬퍼
4. **List 모드** — 아이템 단위 탐색, 선택 강조
5. **Edit / EditValue 모드** — 옵션 간 이동, 값 인라인 편집, 버퍼 가드(i32)
6. **Adding 모드** — 4단계 스테이트 머신, Esc 전체 취소
7. **ConfirmDelete** — 시각 표시, 두 번째 `d`로 삭제, 다른 키로 취소
8. **dirty 트래킹 + 저장** — 상단 표시, flash 메시지, 종료 가드
9. **Undo/Redo** (선택) — 액션 스택
10. **마무리** — clippy, 한글 폭 처리, 백업, README 짧게

## 테스트 전략

`cargo test`는 CLAUDE.md에 적혀 있지만, **TUI 자체는 자동화 테스트가 어렵다**. 그래서 테스트는 *순수 로직*만 단위 테스트로 다루고, UI는 수동 시나리오로 검증한다.

### 자동 테스트 대상 (단위 테스트, `#[cfg(test)] mod tests`)

- `item.rs` — `OptionKind` ↔ 한글 문자열 직렬화 라운드트립, 모든 6종 변형 커버
- `storage.rs` — 빈 파일 로드 시 빈 Vec, 저장 후 재로드 동등성, 잘못된 JSON 에러 처리, 임시 디렉터리 사용
- `app.rs` — 액션 단위 상태 전이 테스트
  - List에서 `Enter` → Edit 진입, `Esc` → List 복귀
  - EditValue에서 `Enter` 적용 → 값 변경 + dirty=true
  - EditValue에서 `Esc` → 값 미변경 + dirty 유지
  - ConfirmDelete에서 두 번째 `d` → 삭제 + dirty=true
  - ConfirmDelete에서 다른 키 → 취소
  - Adding 4단계 정상 흐름, 중간 Esc 시 메모리 미변경
  - (구현 시) Undo가 직전 액션을 되돌림

> 핵심 원칙: 키 입력 → 액션 → 상태 변화를 함수로 분리해 두면, ratatui/crossterm 없이도 상태 머신을 직접 호출해 테스트할 수 있다. `event.rs`가 키를 `Action`으로 매핑하고, `app.rs::apply(action)`이 상태를 바꾸는 구조로 짜면 자연스럽게 테스트 가능.

### 수동 검증 (DoD에서 다룸)

렌더링 결과, 색상, 한글 정렬, 실제 키 반응성 등.

## 완료 조건 (DoD)

- [ ] `cargo build`, `cargo clippy`, `cargo test` 모두 경고/실패 없음
- [ ] 빈 파일에서 시작 → 추가 → 저장 → 재실행 시 복원 확인
- [ ] List → Edit → EditValue 흐름 정상, Esc 단계별 복귀
- [ ] 추가 도중 Esc로 안전한 취소 (메모리 미변경)
- [ ] `d` 한 번 누른 상태가 시각적으로 명확히 표시되고, Esc/다른 키로 해제
- [ ] dirty 시 상단에 `● 저장 안 됨` 표시, 저장 후 사라짐
- [ ] dirty 상태에서 `q` 시 확인 프롬프트
- [ ] periwinkle 강조가 선택/편집/활성 힌트에 일관되게 적용
- [ ] 한글 옵션명 깨짐 없이 정렬 표시

## 리스크 / 메모

- Windows 콘솔에서 한글 폭 계산이 어긋날 수 있음 → `unicode-width` 의존 검토
- crossterm 키 이벤트는 Windows에서 `KeyEventKind::Press`만 필터링해야 중복 입력 방지 (`d` 두 번 눌림 오인 방지에 특히 중요)
- 동일 옵션 종류를 두 번 선택하는 것이 허용됨
- Truecolor 미지원 터미널에서 periwinkle 톤이 크게 달라 보일 수 있음 → 폴백 색 검증
