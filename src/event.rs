use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{Action, AddStep, App, Mode};

pub fn key_to_action(app: &App, key: KeyEvent) -> Option<Action> {
    match &app.mode {
        Mode::List => list_action(key),
        Mode::Edit { .. } => edit_action(key),
        Mode::EditValue { .. } => value_input_action(key),
        Mode::Adding(step) => adding_action(key, step),
        Mode::ConfirmDelete { .. } => confirm_delete_action(key),
        Mode::QuitConfirm => quit_confirm_action(key),
    }
}

fn list_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => Some(Action::Enter),
        KeyCode::Char('a') => Some(Action::AddItem),
        KeyCode::Char('d') => Some(Action::Delete),
        KeyCode::Char('s') => Some(Action::Save),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('u') => Some(Action::Undo),
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Redo),
        _ => None,
    }
}

fn edit_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => Some(Action::Left),
        KeyCode::Right | KeyCode::Char('l') => Some(Action::Right),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Esc => Some(Action::Escape),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        _ => None,
    }
}

fn value_input_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => Some(Action::InputChar(c)),
        KeyCode::Backspace => Some(Action::Backspace),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Esc => Some(Action::Escape),
        _ => None,
    }
}

fn adding_action(key: KeyEvent, step: &AddStep) -> Option<Action> {
    let is_input_step = matches!(
        step,
        AddStep::InputValue1 { .. } | AddStep::InputValue2 { .. }
    );

    if is_input_step {
        return match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => Some(Action::InputChar(c)),
            KeyCode::Backspace => Some(Action::Backspace),
            KeyCode::Enter => Some(Action::Enter),
            KeyCode::Esc => Some(Action::Escape),
            _ => None,
        };
    }

    // SelectKind steps
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Esc => Some(Action::Escape),
        _ => None,
    }
}

fn confirm_delete_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('d') => Some(Action::Delete),
        _ => Some(Action::Escape),
    }
}

fn quit_confirm_action(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('s') => Some(Action::QuitSave),
        KeyCode::Char('q') => Some(Action::QuitForce),
        KeyCode::Esc => Some(Action::Escape),
        _ => None,
    }
}
