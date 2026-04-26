# fix: 아이템 목록 스크롤 위로 이동 시 화면 점프 현상 수정

**커밋**: `56a2227`  
**브랜치**: `main`  
**날짜**: 2026-04-26

---

## 문제

아이템 목록이 화면보다 길어 스크롤이 발생한 상태에서, 위 방향키를 누를 때 포커스만 한 칸 올라가야 하는데 화면 배치까지 맨 위로 당겨지는 현상이 있었음.

예시: 아이템 2~10이 화면에 표시되고 10번에 포커스가 있을 때 위 키를 누르면, 포커스가 9로 이동하면서 화면도 1~9로 재배치됨 (어색함).

## 원인

`scroll_offset`이 초기값 0에서 갱신되지 않아, `compute_offset()` 호출 시 항상 `current_offset=0`을 기준으로 계산됨. 아래로 이동할 때는 `selected`가 뷰 밖을 벗어나므로 offset이 자연히 증가하지만, 위로 이동 시 `selected`가 `current_offset=0` 기준의 뷰 안으로 들어오면서 offset이 0으로 초기화됨.

## 수정 내용

- `render` / `render_main` 함수 시그니처를 `&App` → `&mut App`으로 변경
- 렌더 시 `compute_offset()`이 계산한 offset을 `app.scroll_offset`에 저장
- 이제 이전 렌더의 offset이 다음 렌더의 기준점으로 유지되어, 위 이동 시 포커스가 화면 맨 위에 닿을 때만 스크롤이 발생함

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/ui.rs` | `render`, `render_main` 시그니처를 `&mut App`으로 변경; 계산된 offset을 `app.scroll_offset`에 저장 |
| `src/main.rs` | `ui::render` 호출 시 `&mut app` 전달 |
