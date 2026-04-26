# refactor: 코드 중복 제거 및 불필요한 복사 연산 정리

**커밋**: `7fb14ee`  
**브랜치**: `main`  
**날짜**: 2026-04-26

---

## 변경 동기

이전 커밋에서 `AddFocus::SelectRow` 관련 로직이 추가되면서, `kind_cursor` 위치를 계산하는 코드(`OptionKind::ALL.iter().position(...)`)가 4곳에 흩어져 중복 작성된 상태였음. 또한 이미 소유권을 가진 값에 불필요하게 `.clone()`을 한 번 더 호출하거나, `self.mode`를 if/else 양 갈래에서 동일하게 대입하는 코드가 포함되어 있었음. 기능 자체는 동일하게 유지하면서 이 부분들을 정리함.

## 수정 내용

- **`OptionKind::index_in_all()` 헬퍼 신규 추가** — `OptionKind::ALL`에서 자신의 인덱스를 반환하는 메서드. 기존에 4곳에서 인라인으로 반복되던 패턴을 단일 호출로 통일
- **`EditValue` 핸들러의 이중 clone 제거** — 이미 소유권을 가진 `String`에 `.clone()`을 한 번 더 호출하던 부분을 이동(move)으로 교체
- **`Action::Enter (InputValue)` 의 중복 `self.mode` 대입 제거** — if/else 양 갈래에서 동일한 `self.mode = Mode::Adding(state)` 를 반복하던 부분을 블록 밖으로 끌어냄
- **`val_draft.clear()` 사용** — `val_draft = String::new()` 를 재할당 없이 버퍼를 비우는 방식으로 변경
- **`do_save` / `do_save_stats`의 불필요한 path clone 제거** — `&self.file_path` 를 `&self.file_path.clone()` 으로 넘기던 부분 수정
- **clippy `collapsible_if` 경고 해소** — `handle_edit_value`의 중첩 if를 `if let Ok(...) && ...` 로 통합

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/item.rs` | `OptionKind::index_in_all()` 메서드 추가 |
| `src/app.rs` | 중복 kind_cursor 조회 제거; 불필요한 clone 제거; 중복 mode 대입 제거; clippy 경고 해소 |
