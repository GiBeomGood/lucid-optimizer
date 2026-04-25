use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{AddStep, App, Mode};
use crate::item::OptionKind;

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

    render_main(f, app, chunks[0]);
    render_hint(f, app, chunks[1]);

    match &app.mode {
        Mode::Adding(step) => render_adding_overlay(f, app, step, area),
        Mode::QuitConfirm => render_quit_confirm(f, area),
        _ => {}
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
            format!(" Items ({})", app.items.len()),
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
                format!("{}. ", item_idx + 1),
                Style::default().fg(DANGER),
            )
        } else if is_selected {
            Span::styled(
                format!("{}. ", item_idx + 1),
                Style::default().fg(ACCENT),
            )
        } else {
            Span::raw(format!("{}. ", item_idx + 1))
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
        let indent = Span::raw("      ");
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
            let label_style = if *o == opt_idx {
                Style::default().fg(ACCENT)
            } else {
                Style::default().fg(ACCENT_DIM)
            };
            let val_style = if *o == opt_idx {
                Style::default().fg(ACCENT).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default()
            };
            vec![
                Span::styled(label, label_style),
                Span::styled(opt.value.to_string(), val_style),
            ]
        }
        Mode::EditValue { item_idx: i, option_idx: o, buffer } if *i == item_idx => {
            if *o == opt_idx {
                let box_str = format!("[{}_]", buffer);
                vec![
                    Span::styled(label, Style::default().fg(ACCENT)),
                    Span::styled(
                        box_str,
                        Style::default()
                            .fg(ACCENT)
                            .add_modifier(Modifier::REVERSED),
                    ),
                ]
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

fn render_hint(f: &mut Frame, app: &App, area: Rect) {
    let line = match &app.mode {
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
            ("←→", "옵션 이동"),
            ("Enter", "값 편집"),
            ("↑↓", "복귀+이동"),
            ("Esc", "복귀"),
        ]),
        Mode::EditValue { .. } => hint_line(&[
            ("숫자/-", "입력"),
            ("Backspace", "삭제"),
            ("Enter", "적용"),
            ("Esc", "취소"),
        ]),
        Mode::Adding(step) => {
            let step_label = match step {
                AddStep::SelectKind1 { .. } => "옵션1 선택",
                AddStep::InputValue1 { .. } => "값1 입력",
                AddStep::SelectKind2 { .. } => "옵션2 선택",
                AddStep::InputValue2 { .. } => "값2 입력",
            };
            let mut spans = vec![
                Span::styled("추가 중: ", Style::default().fg(ACCENT)),
                Span::styled(step_label, Style::default().fg(WARN)),
                Span::raw("  "),
            ];
            let extra = match step {
                AddStep::SelectKind1 { .. } | AddStep::SelectKind2 { .. } => {
                    hint_line(&[("↑↓", "선택"), ("Enter", "다음"), ("Esc", "취소")])
                }
                AddStep::InputValue1 { .. } | AddStep::InputValue2 { .. } => {
                    hint_line(&[("숫자", "입력"), ("Enter", "다음"), ("Esc", "취소")])
                }
            };
            spans.extend(extra.spans);
            Line::from(spans)
        }
        Mode::ConfirmDelete { .. } => Line::from(vec![
            Span::styled("한 번 더 ", Style::default().fg(MUTED)),
            Span::styled("d", Style::default().fg(DANGER).add_modifier(Modifier::BOLD)),
            Span::styled(": 삭제 확정", Style::default().fg(WARN)),
            Span::styled("  |  ", Style::default().fg(MUTED)),
            Span::styled("Esc", Style::default().fg(ACCENT)),
            Span::styled(": 취소", Style::default().fg(MUTED)),
        ]),
        Mode::QuitConfirm => Line::from(vec![
            Span::styled("s", Style::default().fg(ACCENT)),
            Span::styled(": 저장 후 종료  ", Style::default().fg(MUTED)),
            Span::styled("q", Style::default().fg(ACCENT)),
            Span::styled(": 그냥 종료  ", Style::default().fg(MUTED)),
            Span::styled("Esc", Style::default().fg(ACCENT)),
            Span::styled(": 취소", Style::default().fg(MUTED)),
        ]),
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

fn render_adding_overlay(f: &mut Frame, _app: &App, step: &AddStep, area: Rect) {
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 12u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect { x, y, width: popup_width, height: popup_height };

    f.render_widget(Clear, popup_area);

    let (title, content_lines) = match step {
        AddStep::SelectKind1 { cursor } => {
            let lines = kind_select_lines(*cursor);
            ("아이템 추가 — 옵션 1 선택", lines)
        }
        AddStep::InputValue1 { kind, buffer } => {
            let lines = value_input_lines(kind, buffer);
            ("아이템 추가 — 옵션 1 값 입력", lines)
        }
        AddStep::SelectKind2 { cursor, .. } => {
            let lines = kind_select_lines(*cursor);
            ("아이템 추가 — 옵션 2 선택", lines)
        }
        AddStep::InputValue2 { kind2, buffer, .. } => {
            let lines = value_input_lines(kind2, buffer);
            ("아이템 추가 — 옵션 2 값 입력", lines)
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let p = Paragraph::new(content_lines).wrap(Wrap { trim: false });
    f.render_widget(p, inner);
}

fn kind_select_lines(cursor: usize) -> Vec<Line<'static>> {
    OptionKind::ALL
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            if i == cursor {
                Line::from(vec![
                    Span::styled(" ▶ ", Style::default().fg(ACCENT)),
                    Span::styled(
                        kind.display_name().to_string(),
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw("   "),
                    Span::raw(kind.display_name().to_string()),
                ])
            }
        })
        .collect()
}

fn value_input_lines(kind: &OptionKind, buffer: &str) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::raw("옵션: "),
            Span::styled(
                kind.display_name().to_string(),
                Style::default().fg(ACCENT),
            ),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::raw("값: "),
            Span::styled(
                format!("[{}_]", buffer),
                Style::default().fg(ACCENT).add_modifier(Modifier::REVERSED),
            ),
        ]),
    ]
}

fn render_quit_confirm(f: &mut Frame, area: Rect) {
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 5u16;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect { x, y, width: popup_width, height: popup_height };

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(WARN))
        .title(Span::styled(
            " 저장 안 된 변경이 있습니다 ",
            Style::default().fg(WARN).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let content = vec![
        Line::from(vec![
            Span::styled("s", Style::default().fg(ACCENT)),
            Span::raw(": 저장 후 종료  "),
            Span::styled("q", Style::default().fg(ACCENT)),
            Span::raw(": 그냥 종료  "),
            Span::styled("Esc", Style::default().fg(ACCENT)),
            Span::raw(": 취소"),
        ]),
    ];
    f.render_widget(Paragraph::new(content).alignment(Alignment::Center), inner);
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
