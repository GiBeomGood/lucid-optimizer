use crate::item::{Item, ItemOption, OptionKind};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddFocus {
    SelectRow,
    SelectKind(u8),
    InputValue(u8),
}

#[derive(Debug, Clone)]
pub struct AddState {
    pub kind1: Option<OptionKind>,
    pub value1: String,
    pub kind2: Option<OptionKind>,
    pub value2: String,
    pub focus: AddFocus,
    pub row_cursor: usize,
    pub kind_cursor: usize,
    pub val_cursor: usize,
}

impl AddState {
    pub fn new() -> Self {
        Self {
            kind1: None,
            value1: String::new(),
            kind2: None,
            value2: String::new(),
            focus: AddFocus::SelectRow,
            row_cursor: 0,
            kind_cursor: 0,
            val_cursor: 0,
        }
    }

    pub fn both_complete(&self) -> bool {
        self.kind1.is_some()
            && self.value1.parse::<i32>().is_ok()
            && !self.value1.is_empty()
            && self.kind2.is_some()
            && self.value2.parse::<i32>().is_ok()
            && !self.value2.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    List,
    Edit { item_idx: usize, option_idx: usize },
    EditValue { item_idx: usize, option_idx: usize, buffer: String, cursor: usize },
    Adding(AddState),
    ConfirmDelete { item_idx: usize },
    QuitConfirm,
}

#[derive(Debug, Clone)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    InputChar(char),
    Backspace,
    Save,
    Delete,
    AddItem,
    Undo,
    Redo,
    Quit,
    QuitForce,
    QuitSave,
}

pub struct App {
    pub items: Vec<Item>,
    pub mode: Mode,
    pub selected: usize,
    pub scroll_offset: usize,
    pub dirty: bool,
    pub flash: Option<(String, Instant)>,
    pub should_quit: bool,
    pub file_path: String,
    undo_stack: Vec<Vec<Item>>,
    redo_stack: Vec<Vec<Item>>,
}

impl App {
    pub fn new(items: Vec<Item>, file_path: String) -> Self {
        Self {
            items,
            mode: Mode::List,
            selected: 0,
            scroll_offset: 0,
            dirty: false,
            flash: None,
            should_quit: false,
            file_path,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        if let Some((_, t)) = &self.flash
            && t.elapsed().as_millis() > 1500
        {
            self.flash = None;
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.items.clone());
        self.redo_stack.clear();
    }

    fn clamp_selected(&mut self) {
        if self.items.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.items.len() - 1);
        }
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Quit => self.handle_quit(),
            Action::QuitForce => self.should_quit = true,
            Action::QuitSave => {
                self.do_save();
                self.should_quit = true;
            }
            Action::Save => self.do_save(),
            Action::Undo => self.do_undo(),
            Action::Redo => self.do_redo(),
            _ => {
                let mode = self.mode.clone();
                match mode {
                    Mode::List => self.handle_list(action),
                    Mode::Edit { item_idx, option_idx } => {
                        self.handle_edit(action, item_idx, option_idx)
                    }
                    Mode::EditValue { item_idx, option_idx, buffer, cursor } => {
                        self.handle_edit_value(action, item_idx, option_idx, buffer, cursor)
                    }
                    Mode::Adding(state) => self.handle_adding(action, state),
                    Mode::ConfirmDelete { item_idx } => {
                        self.handle_confirm_delete(action, item_idx)
                    }
                    Mode::QuitConfirm => self.handle_quit_confirm(action),
                }
            }
        }
    }

    fn handle_quit(&mut self) {
        self.mode = Mode::QuitConfirm;
    }

    fn do_save(&mut self) {
        match crate::storage::save(&self.file_path.clone(), &self.items) {
            Ok(_) => {
                self.dirty = false;
                self.flash = Some(("✓ 저장됨".to_string(), Instant::now()));
            }
            Err(e) => {
                self.flash = Some((format!("저장 실패: {e}"), Instant::now()));
            }
        }
    }

    fn do_undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.items.clone());
            self.items = prev;
            self.dirty = true;
            self.mode = Mode::List;
            self.clamp_selected();
        }
    }

    fn do_redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.items.clone());
            self.items = next;
            self.dirty = true;
            self.mode = Mode::List;
            self.clamp_selected();
        }
    }

    fn handle_list(&mut self, action: Action) {
        match action {
            Action::Up if self.selected > 0 => {
                self.selected -= 1;
            }
            Action::Down if !self.items.is_empty() && self.selected + 1 < self.items.len() => {
                self.selected += 1;
            }
            Action::Enter | Action::Right if !self.items.is_empty() => {
                self.mode = Mode::Edit { item_idx: self.selected, option_idx: 0 };
            }
            Action::AddItem => {
                self.mode = Mode::Adding(AddState::new());
            }
            Action::Delete if !self.items.is_empty() => {
                self.mode = Mode::ConfirmDelete { item_idx: self.selected };
            }
            _ => {}
        }
    }

    fn handle_edit(&mut self, action: Action, item_idx: usize, option_idx: usize) {
        match action {
            Action::Left => {
                self.mode = Mode::Edit { item_idx, option_idx: 0 };
            }
            Action::Right => {
                self.mode = Mode::Edit { item_idx, option_idx: 1 };
            }
            Action::Enter => {
                let buf = self.items[item_idx].options[option_idx].value.to_string();
                let cursor = buf.len();
                self.mode = Mode::EditValue { item_idx, option_idx, buffer: buf, cursor };
            }
            Action::Escape => {
                self.selected = item_idx;
                self.mode = Mode::List;
            }
            Action::Up => {
                self.selected = item_idx;
                if self.selected > 0 {
                    self.selected -= 1;
                }
                self.mode = Mode::List;
            }
            Action::Down => {
                self.selected = item_idx;
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                }
                self.mode = Mode::List;
            }
            _ => {}
        }
    }

    fn handle_edit_value(
        &mut self,
        action: Action,
        item_idx: usize,
        option_idx: usize,
        mut buffer: String,
        mut cursor: usize,
    ) {
        match action {
            Action::Left => {
                cursor = cursor.saturating_sub(1);
                self.mode = Mode::EditValue { item_idx, option_idx, buffer, cursor };
            }
            Action::Right => {
                cursor = (cursor + 1).min(buffer.len());
                self.mode = Mode::EditValue { item_idx, option_idx, buffer, cursor };
            }
            Action::InputChar(c) => {
                if (c == '-' && cursor == 0 && !buffer.starts_with('-')) || c.is_ascii_digit() {
                    buffer.insert(cursor, c);
                    cursor += 1;
                }
                self.mode = Mode::EditValue { item_idx, option_idx, buffer, cursor };
            }
            Action::Backspace => {
                if cursor > 0 {
                    buffer.remove(cursor - 1);
                    cursor -= 1;
                }
                self.mode = Mode::EditValue { item_idx, option_idx, buffer, cursor };
            }
            Action::Enter => {
                if let Ok(val) = buffer.parse::<i32>() {
                    self.push_undo();
                    self.items[item_idx].options[option_idx].value = val;
                    self.dirty = true;
                }
                self.mode = Mode::Edit { item_idx, option_idx };
            }
            Action::Escape => {
                self.mode = Mode::Edit { item_idx, option_idx };
            }
            _ => {}
        }
    }

    fn handle_adding(&mut self, action: Action, mut state: AddState) {
        let focus = state.focus.clone();
        match focus {
            AddFocus::SelectRow => match action {
                Action::Up => {
                    state.row_cursor = state.row_cursor.saturating_sub(1);
                    self.mode = Mode::Adding(state);
                }
                Action::Down => {
                    state.row_cursor = (state.row_cursor + 1).min(1);
                    self.mode = Mode::Adding(state);
                }
                Action::Enter => {
                    if state.both_complete() {
                        self.push_undo();
                        self.items.push(Item {
                            options: [
                                ItemOption {
                                    kind: state.kind1.unwrap(),
                                    value: state.value1.parse().unwrap(),
                                },
                                ItemOption {
                                    kind: state.kind2.unwrap(),
                                    value: state.value2.parse().unwrap(),
                                },
                            ],
                        });
                        self.selected = self.items.len() - 1;
                        self.dirty = true;
                        self.mode = Mode::List;
                    } else {
                        let kind_idx = if state.row_cursor == 0 {
                            state.kind1.and_then(|k| OptionKind::ALL.iter().position(|&x| x == k)).unwrap_or(0)
                        } else {
                            state.kind2.and_then(|k| OptionKind::ALL.iter().position(|&x| x == k)).unwrap_or(0)
                        };
                        state.kind_cursor = kind_idx;
                        state.focus = AddFocus::SelectKind(state.row_cursor as u8);
                        self.mode = Mode::Adding(state);
                    }
                }
                Action::Escape => {
                    self.mode = Mode::List;
                }
                _ => {}
            },
            AddFocus::SelectKind(row) => match action {
                Action::Up => {
                    state.kind_cursor = state.kind_cursor.saturating_sub(1);
                    self.mode = Mode::Adding(state);
                }
                Action::Down => {
                    state.kind_cursor = (state.kind_cursor + 1).min(OptionKind::ALL.len() - 1);
                    self.mode = Mode::Adding(state);
                }
                Action::Enter => {
                    let kind = OptionKind::ALL[state.kind_cursor];
                    let val_cursor = if row == 0 {
                        state.kind1 = Some(kind);
                        state.value1.len()
                    } else {
                        state.kind2 = Some(kind);
                        state.value2.len()
                    };
                    state.val_cursor = val_cursor;
                    state.focus = AddFocus::InputValue(row);
                    self.mode = Mode::Adding(state);
                }
                Action::Escape => {
                    state.focus = AddFocus::SelectRow;
                    self.mode = Mode::Adding(state);
                }
                _ => {}
            },
            AddFocus::InputValue(row) => match action {
                Action::Left => {
                    state.val_cursor = state.val_cursor.saturating_sub(1);
                    self.mode = Mode::Adding(state);
                }
                Action::Right => {
                    let len = if row == 0 { state.value1.len() } else { state.value2.len() };
                    state.val_cursor = (state.val_cursor + 1).min(len);
                    self.mode = Mode::Adding(state);
                }
                Action::InputChar(c) => {
                    let val = if row == 0 { &mut state.value1 } else { &mut state.value2 };
                    if (c == '-' && state.val_cursor == 0 && !val.starts_with('-'))
                        || c.is_ascii_digit()
                    {
                        val.insert(state.val_cursor, c);
                        state.val_cursor += 1;
                    }
                    self.mode = Mode::Adding(state);
                }
                Action::Backspace => {
                    if state.val_cursor > 0 {
                        let val = if row == 0 { &mut state.value1 } else { &mut state.value2 };
                        val.remove(state.val_cursor - 1);
                        state.val_cursor -= 1;
                    }
                    self.mode = Mode::Adding(state);
                }
                Action::Enter => {
                    let valid = if row == 0 {
                        !state.value1.is_empty() && state.value1.parse::<i32>().is_ok()
                    } else {
                        !state.value2.is_empty() && state.value2.parse::<i32>().is_ok()
                    };
                    if valid {
                        state.focus = AddFocus::SelectRow;
                        self.mode = Mode::Adding(state);
                    }
                }
                Action::Escape => {
                    let kind_idx = if row == 0 {
                        state.kind1.and_then(|k| OptionKind::ALL.iter().position(|&x| x == k)).unwrap_or(0)
                    } else {
                        state.kind2.and_then(|k| OptionKind::ALL.iter().position(|&x| x == k)).unwrap_or(0)
                    };
                    state.kind_cursor = kind_idx;
                    state.focus = AddFocus::SelectKind(row);
                    self.mode = Mode::Adding(state);
                }
                _ => {}
            },
        }
    }

    fn handle_confirm_delete(&mut self, action: Action, item_idx: usize) {
        match action {
            Action::Delete => {
                self.push_undo();
                self.items.remove(item_idx);
                self.dirty = true;
                self.clamp_selected();
                self.mode = Mode::List;
            }
            _ => {
                self.mode = Mode::List;
            }
        }
    }

    fn handle_quit_confirm(&mut self, action: Action) {
        match action {
            Action::Save | Action::QuitSave => {
                self.do_save();
                self.should_quit = true;
            }
            Action::Quit | Action::QuitForce => {
                self.should_quit = true;
            }
            Action::Escape => {
                self.mode = Mode::List;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{Item, ItemOption, OptionKind};

    fn make_app() -> App {
        let items = vec![
            Item {
                options: [
                    ItemOption { kind: OptionKind::Magic, value: 10 },
                    ItemOption { kind: OptionKind::CritRate, value: 7 },
                ],
            },
            Item {
                options: [
                    ItemOption { kind: OptionKind::CritDamage, value: 8 },
                    ItemOption { kind: OptionKind::CritRate, value: 13 },
                ],
            },
        ];
        App::new(items, "test.json".to_string())
    }

    #[test]
    fn list_enter_goes_to_edit() {
        let mut app = make_app();
        app.apply(Action::Enter);
        assert!(matches!(app.mode, Mode::Edit { item_idx: 0, option_idx: 0 }));
    }

    #[test]
    fn edit_escape_returns_to_list() {
        let mut app = make_app();
        app.apply(Action::Enter);
        app.apply(Action::Escape);
        assert!(matches!(app.mode, Mode::List));
    }

    #[test]
    fn edit_value_enter_applies_change() {
        let mut app = make_app();
        app.apply(Action::Enter);
        app.apply(Action::Enter);
        // buffer is pre-filled with "10", cursor=2; clear it first
        app.apply(Action::Backspace);
        app.apply(Action::Backspace);
        app.apply(Action::InputChar('9'));
        app.apply(Action::InputChar('9'));
        app.apply(Action::Enter);
        assert_eq!(app.items[0].options[0].value, 99);
        assert!(app.dirty);
    }

    #[test]
    fn edit_value_escape_discards_change() {
        let mut app = make_app();
        let original = app.items[0].options[0].value;
        let original_dirty = app.dirty;
        app.apply(Action::Enter);
        app.apply(Action::Enter);
        app.apply(Action::InputChar('9'));
        app.apply(Action::Escape);
        assert_eq!(app.items[0].options[0].value, original);
        assert_eq!(app.dirty, original_dirty);
    }

    #[test]
    fn edit_value_cursor_movement() {
        let mut app = make_app();
        app.apply(Action::Enter);
        app.apply(Action::Enter); // buffer="10", cursor=2
        app.apply(Action::Left);  // cursor=1
        app.apply(Action::InputChar('5')); // buffer="150", cursor=2
        app.apply(Action::Enter);
        assert_eq!(app.items[0].options[0].value, 150);
    }

    #[test]
    fn confirm_delete_d_removes_item() {
        let mut app = make_app();
        app.apply(Action::Delete);
        app.apply(Action::Delete);
        assert_eq!(app.items.len(), 1);
        assert!(app.dirty);
        assert!(matches!(app.mode, Mode::List));
    }

    #[test]
    fn confirm_delete_escape_cancels() {
        let mut app = make_app();
        app.apply(Action::Delete);
        app.apply(Action::Escape);
        assert_eq!(app.items.len(), 2);
        assert!(matches!(app.mode, Mode::List));
    }

    #[test]
    fn adding_full_flow_appends_item() {
        let mut app = make_app();
        let before = app.items.len();
        app.apply(Action::AddItem);
        // SelectRow, row_cursor=0 → Enter → SelectKind(0)
        app.apply(Action::Enter);
        // SelectKind(0): cursor=0 (Magic) → Enter → InputValue(0)
        app.apply(Action::Enter);
        // InputValue(0): type "5" → Enter → SelectRow
        app.apply(Action::InputChar('5'));
        app.apply(Action::Enter);
        // SelectRow, row_cursor=0 → Down → row_cursor=1 → Enter → SelectKind(1)
        app.apply(Action::Down);
        app.apply(Action::Enter);
        // SelectKind(1): Down → cursor=1 (MagicPercent) → Enter → InputValue(1)
        app.apply(Action::Down);
        app.apply(Action::Enter);
        // InputValue(1): type "3" → Enter → SelectRow
        app.apply(Action::InputChar('3'));
        app.apply(Action::Enter);
        // SelectRow, both complete → Enter → List + item added
        app.apply(Action::Enter);
        assert_eq!(app.items.len(), before + 1);
        assert!(app.dirty);
        assert!(matches!(app.mode, Mode::List));
        let new_item = &app.items[app.items.len() - 1];
        assert_eq!(new_item.options[0].kind, OptionKind::Magic);
        assert_eq!(new_item.options[0].value, 5);
        assert_eq!(new_item.options[1].kind, OptionKind::MagicPercent);
        assert_eq!(new_item.options[1].value, 3);
    }

    #[test]
    fn adding_escape_cancels_without_change() {
        let mut app = make_app();
        let before = app.items.clone();
        app.apply(Action::AddItem); // SelectRow
        app.apply(Action::Escape); // → List
        assert_eq!(app.items, before);
        assert!(matches!(app.mode, Mode::List));
    }

    #[test]
    fn adding_esc_navigates_back() {
        let mut app = make_app();
        app.apply(Action::AddItem);            // SelectRow
        app.apply(Action::Enter);              // SelectKind(0)
        assert!(matches!(app.mode, Mode::Adding(ref s) if matches!(s.focus, AddFocus::SelectKind(0))));
        app.apply(Action::Enter);              // InputValue(0)
        assert!(matches!(app.mode, Mode::Adding(ref s) if matches!(s.focus, AddFocus::InputValue(0))));
        app.apply(Action::Escape);             // back to SelectKind(0)
        assert!(matches!(app.mode, Mode::Adding(ref s) if matches!(s.focus, AddFocus::SelectKind(0))));
        app.apply(Action::Escape);             // back to SelectRow
        assert!(matches!(app.mode, Mode::Adding(ref s) if matches!(s.focus, AddFocus::SelectRow)));
        app.apply(Action::Escape);             // back to List
        assert!(matches!(app.mode, Mode::List));
    }

    #[test]
    fn undo_reverts_last_change() {
        let mut app = make_app();
        app.apply(Action::Delete);
        app.apply(Action::Delete);
        assert_eq!(app.items.len(), 1);
        app.apply(Action::Undo);
        assert_eq!(app.items.len(), 2);
    }

    #[test]
    fn quit_always_goes_to_confirm() {
        let mut app = make_app();
        app.apply(Action::Quit);
        assert!(matches!(app.mode, Mode::QuitConfirm));
    }
}
