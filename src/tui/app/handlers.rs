//! Keyboard event handlers dispatched by mode and panel.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, FormFocus, Mode, Panel};
use crate::tui::input::TextInput;

impl App {
    // ── Helpers ─────────────────────────────────────────────────

    fn cancel_export_dir_edit(&mut self) {
        self.export_dir = TextInput::with_value(&self.export_dir_backup);
        self.editing_export_dir = false;
    }

    // ── Key dispatcher ──────────────────────────────────────────

    /// Top-level key event dispatcher: routes to the handler for the current mode.
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl+C: force quit from anywhere
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Help => self.handle_help_key(key),
            Mode::InitNetwork
            | Mode::AddServer
            | Mode::AddClient
            | Mode::EditServer { .. }
            | Mode::EditClient { .. }
            | Mode::EditNetworkConfig => self.handle_form_key(key),
            Mode::ConfirmDelete { .. } | Mode::ConfirmReinit => self.handle_confirm_key(key),
            Mode::ChooseDeviceType => self.handle_choose_device_type_key(key),
        }
    }

    // ── Normal mode ─────────────────────────────────────────────

    fn handle_normal_key(&mut self, key: KeyEvent) {
        // Global keys (any panel)
        match key.code {
            KeyCode::Char('?') => {
                self.mode = Mode::Help;
                return;
            }
            KeyCode::Tab => {
                if self.editing_export_dir {
                    self.cancel_export_dir_edit();
                }
                self.panel = self.panel.next();
                return;
            }
            KeyCode::BackTab => {
                if self.editing_export_dir {
                    self.cancel_export_dir_edit();
                }
                self.panel = self.panel.prev();
                return;
            }
            _ => {}
        }

        // Ctrl+I: init/reinit network (global)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('i') {
            if self.network_info.is_none() {
                self.force_init = false;
                self.open_init_form();
            } else {
                self.mode = Mode::ConfirmReinit;
            }
            return;
        }

        // Ctrl+E: edit network config (global, requires initialized)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('e') {
            if self.network_info.is_some() {
                self.open_edit_network_form();
            } else {
                self.set_status("Network not initialized. Press CTRL+I to initialize first.");
            }
            return;
        }

        // Panel-specific keys
        match self.panel {
            Panel::OverallInfo => self.handle_info_panel_key(key),
            Panel::DeviceList => self.handle_device_list_key(key),
            Panel::ConfigPreview => self.handle_config_preview_key(key),
        }
    }

    // ── ConfigPreview panel (bottom-right) ──────────────────────

    const fn handle_config_preview_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.config_scroll = self.config_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.config_scroll = self.config_scroll.saturating_add(1);
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    // ── OverallInfo panel (top-left) ────────────────────────────

    fn handle_info_panel_key(&mut self, key: KeyEvent) {
        if self.editing_export_dir {
            // Editing mode: forward keys to export_dir TextInput
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('u') {
                self.export_dir.clear();
                return;
            }
            match key.code {
                KeyCode::Enter => {
                    if self.export_dir.value().trim().is_empty() {
                        self.set_status("Export directory cannot be empty.");
                    } else {
                        self.editing_export_dir = false;
                    }
                }
                KeyCode::Esc => {
                    self.cancel_export_dir_edit();
                }
                KeyCode::Char(c) => self.export_dir.insert(c),
                KeyCode::Backspace => self.export_dir.delete_back(),
                KeyCode::Delete => self.export_dir.delete_forward(),
                KeyCode::Left => self.export_dir.move_left(),
                KeyCode::Right => self.export_dir.move_right(),
                KeyCode::Home => self.export_dir.move_home(),
                KeyCode::End => self.export_dir.move_end(),
                _ => {}
            }
        } else {
            // Non-editing mode: Ctrl+O activates input editing, Q quits
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('o') {
                self.export_dir_backup = self.export_dir.value().to_string();
                self.editing_export_dir = true;
                return;
            }
            if key.code == KeyCode::Char('q') {
                self.should_quit = true;
            }
        }
    }

    // ── DeviceList panel (bottom-left) ──────────────────────────

    fn handle_device_list_key(&mut self, key: KeyEvent) {
        let total = self.device_count();

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.device_index > 0 {
                    self.device_index -= 1;
                    self.update_selected_device();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if total > 0 && self.device_index < total.saturating_sub(1) {
                    self.device_index += 1;
                    self.update_selected_device();
                }
            }
            KeyCode::Home => {
                if self.device_index != 0 {
                    self.device_index = 0;
                    self.update_selected_device();
                }
            }
            KeyCode::End => {
                if total > 0 {
                    let last = total.saturating_sub(1);
                    if self.device_index != last {
                        self.device_index = last;
                        self.update_selected_device();
                    }
                }
            }
            KeyCode::Char('a') => {
                self.mode = Mode::ChooseDeviceType;
            }
            KeyCode::Char('e') => {
                self.open_edit_selected();
            }
            KeyCode::Char('d') => {
                self.open_delete_selected();
            }
            KeyCode::Char('s') => {
                self.generate_and_save_selected();
            }
            KeyCode::Char('g') => {
                self.generate_all();
            }
            KeyCode::PageUp => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
            _ => {}
        }
    }

    // ── ChooseDeviceType mode ───────────────────────────────────

    fn handle_choose_device_type_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s' | 'S') => {
                self.mode = Mode::Normal;
                self.open_add_server_form();
            }
            KeyCode::Char('c' | 'C') => {
                self.mode = Mode::Normal;
                self.open_add_client_form();
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    // ── Generate / Save helpers ─────────────────────────────────

    fn generate_and_save_selected(&mut self) {
        let dir = self.export_dir.value().to_string();
        if dir.is_empty() {
            self.set_status("Export directory is empty. Set it in the top-left panel.");
            return;
        }
        let Some(name) = self.device_name_at(self.device_index).map(str::to_owned) else {
            return;
        };
        match self.manager.generate_config(&name, None) {
            Ok(Some(content)) => {
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    self.set_status(format!("Error creating output dir: {e}"));
                    return;
                }
                let path = format!("{dir}/{name}.conf");
                match std::fs::write(&path, &content) {
                    Ok(()) => self.set_status(format!("Config saved to: {path}")),
                    Err(e) => self.set_status(format!("Error saving: {e}")),
                }
                self.selected_config = Some(content);
                self.selected_device_name = Some(name);
                self.config_scroll = 0;
            }
            Ok(None) => self.set_status("No config content generated"),
            Err(e) => self.set_status(format!("Error: {e}")),
        }
    }

    fn generate_all(&mut self) {
        let dir = self.export_dir.value().to_string();
        if dir.is_empty() {
            self.set_status("Export directory is empty. Set it in the top-left panel.");
            return;
        }
        match self.manager.generate_all_configs(&dir) {
            Ok(count) => {
                self.set_status(format!("Generated {count} config files to: {dir}/"));
            }
            Err(e) => self.set_status(format!("Error: {e}")),
        }
    }

    // ── Form handling (shared by all form modes) ────────────────

    fn handle_form_key(&mut self, key: KeyEvent) {
        let field_count = self.form_fields.len();

        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Tab => {
                self.form_focus = match self.form_focus {
                    FormFocus::Field(i) if i + 1 < field_count => FormFocus::Field(i + 1),
                    FormFocus::Field(_) => FormFocus::Submit,
                    FormFocus::Submit => FormFocus::Cancel,
                    FormFocus::Cancel => FormFocus::Field(0),
                };
            }
            KeyCode::BackTab => {
                self.form_focus = match self.form_focus {
                    FormFocus::Field(0) => FormFocus::Cancel,
                    FormFocus::Field(i) => FormFocus::Field(i.saturating_sub(1)),
                    FormFocus::Submit => FormFocus::Field(field_count.saturating_sub(1)),
                    FormFocus::Cancel => FormFocus::Submit,
                };
            }
            KeyCode::Enter => match self.form_focus {
                FormFocus::Field(i) if i + 1 < field_count => {
                    self.form_focus = FormFocus::Field(i + 1);
                }
                FormFocus::Field(_) => {
                    self.form_focus = FormFocus::Submit;
                }
                FormFocus::Submit => self.submit_form(),
                FormFocus::Cancel => {
                    self.mode = Mode::Normal;
                }
            },
            _ => {
                if let FormFocus::Field(i) = self.form_focus
                    && let Some(field) = self.form_fields.get_mut(i)
                {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('u')
                    {
                        field.clear();
                    } else {
                        match key.code {
                            KeyCode::Char(c) => field.insert(c),
                            KeyCode::Backspace => field.delete_back(),
                            KeyCode::Delete => field.delete_forward(),
                            KeyCode::Left => field.move_left(),
                            KeyCode::Right => field.move_right(),
                            KeyCode::Home => field.move_home(),
                            KeyCode::End => field.move_end(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn submit_form(&mut self) {
        let mode = self.mode.clone();
        match mode {
            Mode::InitNetwork => self.submit_init(),
            Mode::AddServer => self.submit_add_server(),
            Mode::AddClient => self.submit_add_client(),
            Mode::EditServer { name } => self.submit_edit_server(&name),
            Mode::EditClient { name } => self.submit_edit_client(&name),
            Mode::EditNetworkConfig => self.submit_edit_network(),
            _ => {}
        }
    }

    // ── Confirm delete / reinit ─────────────────────────────────

    fn handle_confirm_key(&mut self, key: KeyEvent) {
        let mode = self.mode.clone();
        match key.code {
            KeyCode::Char('y' | 'Y') => match mode {
                Mode::ConfirmDelete { name, device_type } => {
                    let result = if device_type == "server" {
                        self.manager.delete_server(&name)
                    } else {
                        self.manager.delete_client(&name)
                    };
                    match result {
                        Ok(result) => {
                            self.handle_operation_result(&result);
                            self.mode = Mode::Normal;
                            self.refresh_data();
                            self.clamp_device_index();
                            self.update_selected_device();
                        }
                        Err(e) => {
                            self.set_status(format!("Error: {e}"));
                            self.mode = Mode::Normal;
                        }
                    }
                }
                Mode::ConfirmReinit => {
                    self.force_init = true;
                    self.open_init_form();
                }
                _ => {}
            },
            KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    // ── Help ────────────────────────────────────────────────────

    fn handle_help_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?' | 'q') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }
}
