# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 프로젝트 개요

Rust로 작성된 CLI 프로그램. 게임 아이템 정보를 JSON 파일로 관리하고, 아이템 조합 중 최적의 top-k를 찾아주는 두 가지 기능을 제공한다.

아이템 옵션은 6가지 중 하나: 마력, 마력%, 숙련도, 크리티컬 확률, 크리티컬 데미지, 재사용 대기시간 감소. 아이템 하나당 옵션 2개, 각 옵션은 정수 값을 가진다.

## 빌드 및 실행

```bash
cargo build
cargo run
cargo test
cargo test <test_name>   # 단일 테스트 실행
cargo clippy
```

## 기능 구조

**기능 1 — 최적 조합 탐색**: JSON 파일을 읽어 가능한 아이템 조합을 생성하고, 점수 기준으로 top-k를 반환한다.

**기능 2 — TUI 아이템 관리 CLI**: 터미널 UI로 아이템을 추가/수정/삭제한다. 변경 사항은 메모리에 먼저 저장되고, 저장 명령 시 JSON 파일에 반영된다.

## TUI 동작 방식

- 화살표키 또는 `j`/`k`로 아이템 목록 탐색
- 커서 위치에서 좌우 방향키로 옵션 값(`<num>`) 간 이동 및 수정
- 아이템 추가 시: 옵션 선택 → 값 입력 → 옵션 선택 → 값 입력 → 확정
- 화면 하단에 회색 힌트 텍스트 표시 (e.g., `Enter: 값 입력 | D: 삭제`)
- JSON 파일이 비어 있으면 아이템 추가 안내 메시지 표시

## 주요 크레이트

- `ratatui` + `crossterm`: TUI 렌더링 및 키 입력 이벤트 처리. crossterm은 ratatui 백엔드로 쓰이지만, 키 이벤트를 직접 처리할 때 명시적으로 import 필요.
- `serde` + `serde_json`: JSON 직렬화/역직렬화. 구조체에 `#[derive(Serialize, Deserialize)]` 붙이면 자동 처리됨.

## 폰트

한글 표시를 위해 D2Coding 폰트 사용. `assets/D2CodingLigature.ttc` 위치.
