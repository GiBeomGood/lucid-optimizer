use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::app::{AddFocus, AddState, App, Mode};
use crate::item::OptionKind;
use crate::stats::FIELD_NAMES;

// Terminal display width of the widest option name ("재사용 대기시간 감소" = 20 cols).
// All kind names and the placeholder are padded to this so the value column stays aligned.
const KIND_COL_WIDTH: usize = 20;

fn pad_kind(s: &str) -> String {
    let w = UnicodeWidthStr::width(s);
    if w >= KIND_COL_WIDTH {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(KIND_COL_WIDTH - w))
    }
}

const ACCENT: Color = Color::Rgb(180, 170, 255);
const ACCENT_DIM: Color = Color::Rgb(120, 115, 180);
const MUTED: Color = Color::DarkGray;
const WARN: Color = Color::Yellow;
const DANGER: Color = Color::Red;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    if matches!(&app.mode, Mode::QuitConfirm) {
        render_quit_confirm(f, app, area);
        render_hint(f, app, chunks[1]);
        return;
    }

    match &app.mode {
        Mode::Home { cursor } => render_home(f, app, chunks[0], *cursor),
        Mode::Stats { cursor } => render_stats(f, app, chunks[0], *cursor),
        Mode::EditStatValue { field_idx, buffer, cursor } => {
            render_stats_editing(f, app, chunks[0], *field_idx, buffer, *cursor)
        }
        _ => render_main(f, app, chunks[0]),
    }
    render_hint(f, app, chunks[1]);

    if let Mode::Adding(state) = &app.mode {
        render_adding_overlay(f, state, area);
    }
    if let Mode::EditKind { option_idx, kind_cursor, .. } = &app.mode {
        render_edit_kind_overlay(f, *option_idx, *kind_cursor, area);
    }
}

fn render_main(f: &mut Frame, app: &App, area: Rect) {
    let dirty_indicator = if app.dirty {
        Span::styled("● 저장 안 됨", Style::default().fg(WARN))
    } else {
        Span::raw("")
    };

    let flash_span = if let Some((msg, _)) = &app.flash {
        Span::styled(msg.as_str(), Style::default().fg(Color::Green))
    } else {
        dirty_indicator
    };

    let title_line = Line::from(vec![
        Span::styled(
            format!(" 아이템 목록(총 {:>2}개)", app.items.len()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        flash_span,
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title_line);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.items.is_empty() {
        let empty_line = Line::from(vec![
            Span::raw("아이템이 없습니다. "),
            Span::styled("a", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw("를 눌러 추가하세요."),
        ]);
        let p = Paragraph::new(empty_line).alignment(Alignment::Center);
        let centered = Rect {
            x: inner.x,
            y: inner.y + inner.height / 2,
            width: inner.width,
            height: 1,
        };
        f.render_widget(p, centered);
        return;
    }

    // Each item occupies 2 lines
    let visible = inner.height as usize / 2;
    let offset = compute_offset(app.selected, app.scroll_offset, visible);

    for (display_idx, item_idx) in (offset..).take(visible).enumerate() {
        if item_idx >= app.items.len() {
            break;
        }
        let item = &app.items[item_idx];
        let row_y = inner.y + (display_idx * 2) as u16;
        if row_y + 1 >= inner.y + inner.height {
            break;
        }

        let is_selected = item_idx == app.selected;
        let is_confirm_delete = matches!(&app.mode, Mode::ConfirmDelete { item_idx: idx } if *idx == item_idx);

        // Arrow indicator
        let arrow = if is_selected {
            if is_confirm_delete {
                Span::styled(" ▶ ", Style::default().fg(DANGER))
            } else {
                Span::styled(" ▶ ", Style::default().fg(ACCENT))
            }
        } else {
            Span::raw("   ")
        };

        let num_span = if is_confirm_delete {
            Span::styled(
                format!("{:2}. ", item_idx + 1),
                Style::default().fg(DANGER),
            )
        } else if is_selected {
            Span::styled(
                format!("{:2}. ", item_idx + 1),
                Style::default().fg(ACCENT),
            )
        } else {
            Span::raw(format!("{:2}. ", item_idx + 1))
        };

        // Option 0 row
        let opt0_spans = build_option_spans(app, item_idx, 0, &item.options[0]);
        let mut line0_spans = vec![arrow, num_span];
        line0_spans.extend(opt0_spans);

        // ConfirmDelete inline hint
        if is_confirm_delete {
            line0_spans.push(Span::styled(
                "  (삭제하려면 d 한 번 더)",
                Style::default().fg(WARN),
            ));
        }

        let line0 = Line::from(line0_spans);
        f.render_widget(Paragraph::new(line0), Rect { x: inner.x, y: row_y, width: inner.width, height: 1 });

        // Option 1 row
        let opt1_spans = build_option_spans(app, item_idx, 1, &item.options[1]);
        let indent = Span::raw("       ");
        let mut line1_spans = vec![indent];
        line1_spans.extend(opt1_spans);
        let line1 = Line::from(line1_spans);
        f.render_widget(
            Paragraph::new(line1),
            Rect { x: inner.x, y: row_y + 1, width: inner.width, height: 1 },
        );
    }
}

fn build_option_spans<'a>(
    app: &'a App,
    item_idx: usize,
    opt_idx: usize,
    opt: &'a crate::item::ItemOption,
) -> Vec<Span<'a>> {
    let label = format!("{}: ", opt.kind.display_name());

    match &app.mode {
        Mode::Edit { item_idx: i, option_idx: o } if *i == item_idx => {
            let is_focused_row = *o == opt_idx;
            let style = if is_focused_row {
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(ACCENT_DIM)
            };
            vec![
                Span::styled(label, style),
                Span::styled(opt.value.to_string(), style),
            ]
        }
        Mode::EditKind { item_idx: i, option_idx: o, .. } if *i == item_idx => {
            let is_focused_row = *o == opt_idx;
            let style = if is_focused_row {
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(ACCENT_DIM)
            };
            vec![
                Span::styled(label, style),
                Span::styled(opt.value.to_string(), style),
            ]
        }
        Mode::EditValue { item_idx: i, option_idx: o, buffer, cursor } if *i == item_idx => {
            if *o == opt_idx {
                let mut spans = vec![Span::styled(label, Style::default().fg(ACCENT))];
                spans.extend(cursor_spans(buffer, *cursor));
                spans
            } else {
                vec![
                    Span::styled(label, Style::default().fg(ACCENT_DIM)),
                    Span::raw(opt.value.to_string()),
                ]
            }
        }
        _ => vec![Span::raw(label), Span::raw(opt.value.to_string())],
    }
}

fn cursor_spans(buf: &str, cursor: usize) -> Vec<Span<'static>> {
    if cursor >= buf.len() {
        vec![
            Span::raw(buf.to_string()),
            Span::styled(" ", Style::default().add_modifier(Modifier::REVERSED)),
        ]
    } else {
        vec![
            Span::raw(buf[..cursor].to_string()),
            Span::styled(
                buf[cursor..cursor + 1].to_string(),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::raw(buf[cursor + 1..].to_string()),
        ]
    }
}

fn render_hint(f: &mut Frame, app: &App, area: Rect) {
    let line = match &app.mode {
        Mode::Home { .. } => hint_line(&[
            ("↑↓", "이동"),
            ("Enter", "선택"),
            ("q", "종료"),
        ]),
        Mode::Stats { .. } => hint_line(&[
            ("↑↓", "이동"),
            ("Enter", "편집"),
            ("s", "저장"),
            ("Esc", "뒤로"),
        ]),
        Mode::EditStatValue { .. } => hint_line(&[
            ("숫자/-", "입력"),
            ("←→", "커서"),
            ("Backspace", "삭제"),
            ("Enter", "적용"),
            ("Esc", "뒤로"),
        ]),
        Mode::List => hint_line(&[
            ("↑↓", "이동"),
            ("Enter", "편집"),
            ("a", "추가"),
            ("d", "삭제"),
            ("s", "저장"),
            ("u", "되돌리기"),
            ("q", "종료"),
        ]),
        Mode::Edit { .. } => hint_line(&[
            ("↑↓", "행 이동"),
            ("o", "옵션 변경"),
            ("v / Enter", "값 편집"),
            ("Esc", "복귀"),
        ]),
        Mode::EditKind { .. } => hint_line(&[
            ("↑↓", "선택"),
            ("Enter", "확정"),
            ("Esc", "뒤로"),
        ]),
        Mode::EditValue { .. } => hint_line(&[
            ("숫자/-", "입력"),
            ("←→", "커서"),
            ("Backspace", "삭제"),
            ("Enter", "적용"),
            ("Esc", "뒤로"),
        ]),
        Mode::Adding(state) => match state.focus {
            AddFocus::SelectRow => {
                let mut spans = vec![
                    Span::styled("추가 중  ", Style::default().fg(ACCENT)),
                ];
                let extra = if state.both_complete() {
                    hint_line(&[("↑↓", "이동"), ("Enter", "완료"), ("Esc", "취소")])
                } else {
                    hint_line(&[("↑↓", "이동"), ("Enter", "옵션 설정"), ("Esc", "취소")])
                };
                spans.extend(extra.spans);
                Line::from(spans)
            }
            AddFocus::SelectKind(_) => hint_line(&[
                ("↑↓", "선택"),
                ("Enter", "확정"),
                ("Esc", "뒤로"),
            ]),
            AddFocus::InputValue(_) => hint_line(&[
                ("숫자", "입력"),
                ("←→", "커서"),
                ("Enter", "확정"),
                ("Esc", "뒤로"),
            ]),
        },
        Mode::ConfirmDelete { .. } => Line::from(vec![
            Span::styled("한 번 더 ", Style::default().fg(MUTED)),
            Span::styled("d", Style::default().fg(DANGER).add_modifier(Modifier::BOLD)),
            Span::styled(": 삭제 확정", Style::default().fg(WARN)),
            Span::styled("  |  ", Style::default().fg(MUTED)),
            Span::styled("Esc", Style::default().fg(ACCENT)),
            Span::styled(": 취소", Style::default().fg(MUTED)),
        ]),
        Mode::QuitConfirm => {
            if app.is_any_dirty() {
                Line::from(vec![
                    Span::styled("s", Style::default().fg(ACCENT)),
                    Span::styled(": 저장 후 종료  ", Style::default().fg(MUTED)),
                    Span::styled("q", Style::default().fg(ACCENT)),
                    Span::styled(": 그냥 종료  ", Style::default().fg(MUTED)),
                    Span::styled("Esc", Style::default().fg(ACCENT)),
                    Span::styled(": 취소", Style::default().fg(MUTED)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("q", Style::default().fg(ACCENT)),
                    Span::styled(": 종료  ", Style::default().fg(MUTED)),
                    Span::styled("Esc", Style::default().fg(ACCENT)),
                    Span::styled(": 취소", Style::default().fg(MUTED)),
                ])
            }
        }
    };
    f.render_widget(Paragraph::new(line), area);
}

fn hint_line(pairs: &[(&str, &str)]) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, (key, desc)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  |  ", Style::default().fg(MUTED)));
        }
        spans.push(Span::styled(key.to_string(), Style::default().fg(ACCENT)));
        spans.push(Span::styled(
            format!(": {desc}"),
            Style::default().fg(MUTED),
        ));
    }
    Line::from(spans)
}

fn render_adding_overlay(f: &mut Frame, state: &AddState, area: Rect) {
    // Fixed height: 2 option rows + blank + "선택:" label + 6 kind rows = 10 inner lines
    let popup_height = 12u16;
    let popup_width = 54u16.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect { x, y, width: popup_width, height: popup_height };

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .title(Span::styled(
            " 아이템 추가 ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(build_add_option_row(state, 1));
    lines.push(build_add_option_row(state, 2));
    lines.push(Line::from(vec![]));

    let show_list = matches!(state.focus, AddFocus::SelectKind(_));

    if show_list {
        let row = match state.focus { AddFocus::SelectKind(r) => r, _ => 0 };
        let section_label = if row == 0 { "옵션 1 선택:" } else { "옵션 2 선택:" };
        lines.push(Line::from(vec![
            Span::styled(format!("  {section_label}"), Style::default().fg(MUTED)),
        ]));
        for (i, kind) in OptionKind::ALL.iter().enumerate() {
            if i == state.kind_cursor {
                lines.push(Line::from(vec![
                    Span::styled("   ▶ ", Style::default().fg(ACCENT)),
                    Span::styled(
                        kind.display_name().to_string(),
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::raw(kind.display_name().to_string()),
                ]));
            }
        }
    } else {
        // Keep fixed height by padding with empty lines
        for _ in 0..7 {
            lines.push(Line::from(vec![]));
        }
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn build_add_option_row(state: &AddState, opt_num: u8) -> Line<'static> {
    let row = (opt_num - 1) as usize;

    let is_active = match &state.focus {
        AddFocus::SelectRow => state.row_cursor == row,
        AddFocus::SelectKind(r) => *r as usize == row,
        AddFocus::InputValue(r) => *r as usize == row,
    };
    let is_value_inputting = matches!(&state.focus, AddFocus::InputValue(r) if *r as usize == row);

    let arrow = if is_active {
        Span::styled(" ▶ ", Style::default().fg(ACCENT))
    } else {
        Span::raw("   ")
    };

    let label_style = if is_active {
        Style::default().fg(ACCENT)
    } else {
        Style::default().fg(MUTED)
    };

    let label = Span::styled(format!("{opt_num}. "), label_style);

    let (kind, buf) = match opt_num {
        1 => (&state.kind1, state.value1.as_str()),
        2 => (&state.kind2, state.value2.as_str()),
        _ => return Line::default(),
    };

    let kind_span = match kind {
        Some(k) => Span::styled(pad_kind(k.display_name()), if is_active {
            Style::default().fg(ACCENT)
        } else {
            Style::default()
        }),
        None => Span::styled(pad_kind("<옵션 선택>"), Style::default().fg(MUTED)),
    };

    let mut spans = vec![arrow, label, kind_span, Span::raw("    ")];

    if is_value_inputting {
        spans.extend(cursor_spans(buf, state.val_cursor));
    } else if kind.is_some() && !buf.is_empty() {
        spans.push(Span::raw(buf.to_string()));
    } else {
        spans.push(Span::styled("<값 입력>", Style::default().fg(MUTED)));
    }

    Line::from(spans)
}

fn render_home(f: &mut Frame, app: &App, area: Rect, cursor: usize) {
    let flash_span = if let Some((msg, _)) = &app.flash {
        Span::styled(msg.as_str(), Style::default().fg(Color::Green))
    } else {
        Span::raw("")
    };

    let title_line = Line::from(vec![
        Span::styled(" Lucid Optimizer ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        flash_span,
    ]);

    let block = Block::default().borders(Borders::ALL).title(title_line);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let menu_items = ["기본 정보 보기", "아이템 목록 보기"];
    let total_height = menu_items.len() as u16;
    let start_y = inner.y + inner.height.saturating_sub(total_height) / 2;

    for (i, label) in menu_items.iter().enumerate() {
        let y = start_y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }
        let (arrow, style) = if i == cursor {
            (Span::styled(" ▶ ", Style::default().fg(ACCENT)), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        } else {
            (Span::raw("   "), Style::default())
        };
        let line = Line::from(vec![arrow, Span::styled(label.to_string(), style)]);
        f.render_widget(Paragraph::new(line), Rect { x: inner.x, y, width: inner.width, height: 1 });
    }
}

fn render_stats(f: &mut Frame, app: &App, area: Rect, cursor: usize) {
    render_stats_inner(f, app, area, None, None, cursor);
}

fn render_stats_editing(f: &mut Frame, app: &App, area: Rect, field_idx: usize, buffer: &str, val_cursor: usize) {
    render_stats_inner(f, app, area, Some(field_idx), Some((buffer, val_cursor)), field_idx);
}

fn render_stats_inner(
    f: &mut Frame,
    app: &App,
    area: Rect,
    editing: Option<usize>,
    edit_buf: Option<(&str, usize)>,
    cursor: usize,
) {
    let dirty_span = if app.stats_dirty {
        Span::styled("● 저장 안 됨", Style::default().fg(WARN))
    } else {
        Span::raw("")
    };

    let flash_span = if let Some((msg, _)) = &app.flash {
        Span::styled(msg.as_str(), Style::default().fg(Color::Green))
    } else {
        dirty_span
    };

    let title_line = Line::from(vec![
        Span::styled(" 기본 정보 ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        flash_span,
    ]);

    let block = Block::default().borders(Borders::ALL).title(title_line);
    let inner = block.inner(area);
    f.render_widget(block, area);

    for (i, name) in FIELD_NAMES.iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let is_cursor = i == cursor;
        let is_editing = editing == Some(i);

        let arrow = if is_cursor {
            Span::styled(" ▶ ", Style::default().fg(ACCENT))
        } else {
            Span::raw("   ")
        };

        let label = format!("{}: ", pad_kind(name));
        let label_style = if is_cursor {
            Style::default().fg(ACCENT)
        } else {
            Style::default()
        };

        let mut spans = vec![arrow, Span::styled(label, label_style)];

        if is_editing {
            let (buf, val_cursor) = edit_buf.unwrap();
            spans.extend(cursor_spans(buf, val_cursor));
        } else {
            let val = app.stats.get(i).to_string();
            let val_style = if is_cursor {
                Style::default().fg(ACCENT).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default()
            };
            spans.push(Span::styled(val, val_style));
        }

        let line = Line::from(spans);
        f.render_widget(Paragraph::new(line), Rect { x: inner.x, y, width: inner.width, height: 1 });
    }
}

fn render_quit_confirm(f: &mut Frame, app: &App, area: Rect) {
    f.render_widget(Clear, area);

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT_DIM))
        .title(Span::styled(
            " Lucid Optimizer ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(outer_block, area);

    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 5u16;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect { x, y, width: popup_width, height: popup_height };

    f.render_widget(Clear, popup_area);

    let (border_color, title) = if app.is_any_dirty() {
        (WARN, " 저장 안 된 변경이 있습니다 ")
    } else {
        (ACCENT, " 종료 확인 ")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            title,
            Style::default().fg(border_color).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let content = if app.is_any_dirty() {
        vec![Line::from(vec![
            Span::styled("s", Style::default().fg(ACCENT)),
            Span::raw(": 저장 후 종료  "),
            Span::styled("q", Style::default().fg(ACCENT)),
            Span::raw(": 그냥 종료  "),
            Span::styled("Esc", Style::default().fg(ACCENT)),
            Span::raw(": 취소"),
        ])]
    } else {
        vec![Line::from(vec![
            Span::styled("q", Style::default().fg(ACCENT)),
            Span::raw(": 종료  "),
            Span::styled("Esc", Style::default().fg(ACCENT)),
            Span::raw(": 취소"),
        ])]
    };
    f.render_widget(Paragraph::new(content).alignment(Alignment::Center), inner);
}

fn render_edit_kind_overlay(f: &mut Frame, option_idx: usize, kind_cursor: usize, area: Rect) {
    let popup_height = (OptionKind::ALL.len() as u16) + 4;
    let popup_width = 32u16.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect { x, y, width: popup_width, height: popup_height };

    f.render_widget(Clear, popup_area);

    let title = if option_idx == 0 { " 옵션 1 변경 " } else { " 옵션 2 변경 " };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .title(Span::styled(title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let mut lines: Vec<Line<'static>> = Vec::new();
    for (i, kind) in OptionKind::ALL.iter().enumerate() {
        if i == kind_cursor {
            lines.push(Line::from(vec![
                Span::styled(" ▶ ", Style::default().fg(ACCENT)),
                Span::styled(kind.display_name().to_string(), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::raw(kind.display_name().to_string()),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn compute_offset(selected: usize, current_offset: usize, visible: usize) -> usize {
    if visible == 0 {
        return 0;
    }
    if selected < current_offset {
        selected
    } else if selected >= current_offset + visible {
        selected + 1 - visible
    } else {
        current_offset
    }
}
