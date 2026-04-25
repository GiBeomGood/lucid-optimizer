use crate::item::{Item, ItemOption, OptionKind};
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum AddStep {
    SelectKind1 { cursor: usize },
    InputValue1 { kind: OptionKind, buffer: String },
    SelectKind2 { kind1: OptionKind, value1: i32, cursor: usize },
    InputValue2 { kind1: OptionKind, value1: i32, kind2: OptionKind, buffer: String },
}

#[derive(Debug, Clone)]
pub enum Mode {
    List,
    Edit { item_idx: usize, option_idx: usize },
    EditValue { item_idx: usize, option_idx: usize, buffer: String },
    Adding(AddStep),
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
                    Mode::EditValue { item_idx, option_idx, buffer } => {
                        self.handle_edit_value(action, item_idx, option_idx, buffer)
                    }
                    Mode::Adding(step) => self.handle_adding(action, step),
                    Mode::ConfirmDelete { item_idx } => {
                        self.handle_confirm_delete(action, item_idx)
                    }
                    Mode::QuitConfirm => self.handle_quit_confirm(action),
                }
            }
        }
    }

    fn handle_quit(&mut self) {
        if self.dirty {
            self.mode = Mode::QuitConfirm;
        } else {
            self.should_quit = true;
        }
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
                self.mode = Mode::Adding(AddStep::SelectKind1 { cursor: 0 });
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
                self.mode = Mode::EditValue { item_idx, option_idx, buffer: buf };
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
    ) {
        match action {
            Action::InputChar(c) => {
                if (c == '-' && buffer.is_empty()) || c.is_ascii_digit() {
                    buffer.push(c);
                }
                self.mode = Mode::EditValue { item_idx, option_idx, buffer };
            }
            Action::Backspace => {
                buffer.pop();
                self.mode = Mode::EditValue { item_idx, option_idx, buffer };
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

    fn handle_adding(&mut self, action: Action, step: AddStep) {
        match step {
            AddStep::SelectKind1 { cursor } => match action {
                Action::Up => {
                    let c = if cursor == 0 { 0 } else { cursor - 1 };
                    self.mode = Mode::Adding(AddStep::SelectKind1 { cursor: c });
                }
                Action::Down => {
                    let c = (cursor + 1).min(OptionKind::ALL.len() - 1);
                    self.mode = Mode::Adding(AddStep::SelectKind1 { cursor: c });
                }
                Action::Enter => {
                    let kind = OptionKind::ALL[cursor];
                    self.mode =
                        Mode::Adding(AddStep::InputValue1 { kind, buffer: String::new() });
                }
                Action::Escape => {
                    self.mode = Mode::List;
                }
                _ => {}
            },
            AddStep::InputValue1 { kind, mut buffer } => match action {
                Action::InputChar(c) => {
                    if (c == '-' && buffer.is_empty()) || c.is_ascii_digit() {
                        buffer.push(c);
                    }
                    self.mode = Mode::Adding(AddStep::InputValue1 { kind, buffer });
                }
                Action::Backspace => {
                    buffer.pop();
                    self.mode = Mode::Adding(AddStep::InputValue1 { kind, buffer });
                }
                Action::Enter => {
                    if let Ok(value1) = buffer.parse::<i32>() {
                        self.mode = Mode::Adding(AddStep::SelectKind2 {
                            kind1: kind,
                            value1,
                            cursor: 0,
                        });
                    }
                }
                Action::Escape => {
                    self.mode = Mode::List;
                }
                _ => {}
            },
            AddStep::SelectKind2 { kind1, value1, cursor } => match action {
                Action::Up => {
                    let c = if cursor == 0 { 0 } else { cursor - 1 };
                    self.mode = Mode::Adding(AddStep::SelectKind2 { kind1, value1, cursor: c });
                }
                Action::Down => {
                    let c = (cursor + 1).min(OptionKind::ALL.len() - 1);
                    self.mode = Mode::Adding(AddStep::SelectKind2 { kind1, value1, cursor: c });
                }
                Action::Enter => {
                    let kind2 = OptionKind::ALL[cursor];
                    self.mode = Mode::Adding(AddStep::InputValue2 {
                        kind1,
                        value1,
                        kind2,
                        buffer: String::new(),
                    });
                }
                Action::Escape => {
                    self.mode = Mode::List;
                }
                _ => {}
            },
            AddStep::InputValue2 { kind1, value1, kind2, mut buffer } => match action {
                Action::InputChar(c) => {
                    if (c == '-' && buffer.is_empty()) || c.is_ascii_digit() {
                        buffer.push(c);
                    }
                    self.mode = Mode::Adding(AddStep::InputValue2 { kind1, value1, kind2, buffer });
                }
                Action::Backspace => {
                    buffer.pop();
                    self.mode = Mode::Adding(AddStep::InputValue2 { kind1, value1, kind2, buffer });
                }
                Action::Enter => {
                    if let Ok(value2) = buffer.parse::<i32>() {
                        self.push_undo();
                        self.items.push(Item {
                            options: [
                                ItemOption { kind: kind1, value: value1 },
                                ItemOption { kind: kind2, value: value2 },
                            ],
                        });
                        self.selected = self.items.len() - 1;
                        self.dirty = true;
                        self.mode = Mode::List;
                    }
                }
                Action::Escape => {
                    self.mode = Mode::List;
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
        // clear existing buffer ("10") then type "99"
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
        // SelectKind1: choose index 0 (Magic)
        app.apply(Action::Enter);
        // InputValue1: type "5"
        app.apply(Action::InputChar('5'));
        app.apply(Action::Enter);
        // SelectKind2: choose index 1 (MagicPercent)
        app.apply(Action::Down);
        app.apply(Action::Enter);
        // InputValue2: type "3"
        app.apply(Action::InputChar('3'));
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
        app.apply(Action::AddItem);
        app.apply(Action::Enter);
        app.apply(Action::InputChar('5'));
        app.apply(Action::Escape);
        assert_eq!(app.items, before);
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
}
