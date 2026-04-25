# lucid-optimizer

게임 아이템 정보를 관리하고, 최적의 아이템 조합을 찾아주는 Rust CLI 프로그램.

## 기능

### 기능 1 — 최적 조합 탐색

JSON 파일에 저장된 아이템 데이터를 읽어 가능한 모든 조합을 생성하고, 점수 기준으로 top-k 조합을 반환한다.

### 기능 2 — TUI 아이템 관리

`ratatui` 기반 터미널 UI로 아이템을 추가/수정/삭제한다. 변경 사항은 메모리에 먼저 반영되고, 저장 명령 시 JSON 파일에 기록된다.

## 아이템 구조

아이템 하나는 옵션 2개로 구성되며, 각 옵션은 아래 6가지 중 하나이고 정수 값을 가진다.

| 옵션 |
|------|
| 마력 |
| 마력% |
| 숙련도 |
| 크리티컬 확률 |
| 크리티컬 데미지 |
| 재사용 대기시간 감소 |

**JSON 예시:**

```json
{
  "크리티컬 데미지": 8,
  "크리티컬 확률": 13
}
```

## TUI 조작법

| 키 | 동작 |
|----|------|
| `↑` / `k` | 목록 위로 이동 |
| `↓` / `j` | 목록 아래로 이동 |
| `←` / `→` | 현재 아이템의 옵션 값 간 이동 |
| `Enter` | 값 입력 |
| `d` | 아이템 삭제 |

- 아이템 추가 흐름: 옵션 선택 → 값 입력 → 옵션 선택 → 값 입력 → 확정 (진행 중 취소 가능)
- JSON이 비어 있으면 아이템 추가 안내 메시지 표시
- 화면 하단에 회색 힌트 텍스트 표시

## 빌드 및 실행

```bash
cargo build
cargo run
cargo test
cargo clippy
```

## 의존성

- [`ratatui`](https://github.com/ratatui-org/ratatui) + [`crossterm`](https://github.com/crossterm-rs/crossterm) — TUI 렌더링 및 키 입력 처리
- [`serde`](https://serde.rs/) + [`serde_json`](https://github.com/serde-rs/json) — JSON 직렬화/역직렬화

## 기타

한글 표시를 위해 D2Coding 폰트(`assets/D2CodingLigature.ttc`)를 사용한다.
