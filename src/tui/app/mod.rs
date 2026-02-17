//! Application state, panel/mode enums, and cached data management for the TUI.

use crate::manager::{
    ClientEntry, DeviceInfo, NetworkInfo, OperationResult, ServerEntry, WireGuardManager,
};
use crate::tui::input::TextInput;
use chrono::Local;

mod forms;
mod handlers;

#[cfg(test)]
mod tests;

// ── Enums ───────────────────────────────────────────────────────

/// Active panel in the TUI (focusable panels participate in Tab cycling).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    OverallInfo,   // top-left (editable: export dir)
    DeviceList,    // bottom-left (device navigation & actions)
    ConfigPreview, // bottom-right (scrollable config preview)
}

impl Panel {
    /// Cycle to the next focusable panel.
    pub const fn next(self) -> Self {
        match self {
            Self::OverallInfo => Self::DeviceList,
            Self::DeviceList => Self::ConfigPreview,
            Self::ConfigPreview => Self::OverallInfo,
        }
    }

    /// Cycle to the previous focusable panel.
    pub const fn prev(self) -> Self {
        match self {
            Self::OverallInfo => Self::ConfigPreview,
            Self::DeviceList => Self::OverallInfo,
            Self::ConfigPreview => Self::DeviceList,
        }
    }
}

/// Current interaction mode (state machine).
#[derive(Clone)]
pub enum Mode {
    Normal,
    InitNetwork,
    AddServer,
    AddClient,
    EditServer { name: String },
    EditClient { name: String },
    ConfirmDelete { name: String, device_type: String },
    ConfirmReinit,
    EditNetworkConfig,
    ChooseDeviceType, // new: choose server or client when adding
    Help,
}

/// Focus position within a form modal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormFocus {
    Field(usize),
    Submit,
    Cancel,
}

// ── App state ───────────────────────────────────────────────────

/// Top-level application state for the TUI.
pub struct App {
    pub manager: WireGuardManager,
    pub panel: Panel,
    pub mode: Mode,
    pub should_quit: bool,

    // Cached data
    pub servers: Vec<ServerEntry>,
    pub clients: Vec<ClientEntry>,
    pub network_info: Option<NetworkInfo>,

    // Unified device index (servers first, then clients)
    pub device_index: usize,

    // Form state
    pub form_fields: Vec<TextInput>,
    pub form_labels: Vec<&'static str>,
    pub form_focus: FormFocus,

    // Status message
    pub status: String,

    // Whether init should use force mode (reinit)
    pub force_init: bool,

    // Export directory input (persistent, shown in OverallInfo panel)
    pub export_dir: TextInput,
    // Whether the export dir input is actively being edited
    pub editing_export_dir: bool,
    // Saved value before editing (for Esc restore)
    pub export_dir_backup: String,

    // Right-side panel caches (read-only, auto-updated)
    pub selected_device_info: Option<DeviceInfo>,
    pub selected_config: Option<String>,
    pub selected_device_name: Option<String>,

    // Device detail scroll offset
    pub detail_scroll: u16,
    // Config preview scroll offset
    pub config_scroll: u16,

    // Operation log entries: (timestamp, message)
    pub logs: Vec<(String, String)>,
}

impl App {
    /// Create a new `App` with a default manager and initial state.
    pub fn new() -> Self {
        let manager = WireGuardManager::new("data/data.json");
        let mut app = Self {
            manager,
            panel: Panel::OverallInfo,
            mode: Mode::Normal,
            should_quit: false,
            servers: Vec::new(),
            clients: Vec::new(),
            network_info: None,
            device_index: 0,
            form_fields: Vec::new(),
            form_labels: Vec::new(),
            form_focus: FormFocus::Field(0),
            status: String::new(),
            force_init: false,
            export_dir: TextInput::with_value("output"),
            editing_export_dir: false,
            export_dir_backup: String::from("output"),
            selected_device_info: None,
            selected_config: None,
            selected_device_name: None,
            detail_scroll: 0,
            config_scroll: 0,
            logs: Vec::new(),
        };
        app.refresh_data();
        app
    }

    /// Reload server/client lists and network info from the manager.
    pub fn refresh_data(&mut self) {
        self.network_info = self.manager.show_network_config().ok();
        if let Ok((servers, clients)) = self.manager.list_devices() {
            self.servers = servers.values().cloned().collect();
            self.clients = clients.values().cloned().collect();
        }
        self.update_selected_device();
    }

    /// Set the status bar text and append to the operation log.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();
        self.logs.push((timestamp, msg.clone()));
        self.status = msg;
    }

    /// Extract success text from an `OperationResult` into the status bar and log all messages.
    pub fn handle_operation_result(&mut self, result: &OperationResult) {
        if let Some(text) = result.success_text() {
            self.status = text.to_string();
        }
        let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();
        for msg in &result.messages {
            self.logs.push((timestamp.clone(), msg.text.clone()));
        }
    }

    /// Total device count (servers + clients).
    pub const fn device_count(&self) -> usize {
        self.servers.len() + self.clients.len()
    }

    /// Get the device name at the given unified index.
    pub fn device_name_at(&self, index: usize) -> Option<&str> {
        if index < self.servers.len() {
            self.servers.get(index).map(|s| s.name.as_str())
        } else {
            self.clients
                .get(index.saturating_sub(self.servers.len()))
                .map(|c| c.name.as_str())
        }
    }

    /// Whether the device at the given unified index is a server.
    pub const fn is_server_at(&self, index: usize) -> bool {
        index < self.servers.len()
    }

    /// Update the right-side panel caches based on the current `device_index`.
    pub fn update_selected_device(&mut self) {
        self.detail_scroll = 0;
        self.config_scroll = 0;
        let name = self.device_name_at(self.device_index).map(str::to_owned);
        if let Some(ref n) = name {
            self.selected_device_name = Some(n.clone());
            self.selected_device_info = self.manager.show_device(n).ok();
            self.selected_config = self.manager.generate_config(n, None).ok().flatten();
        } else {
            self.selected_device_name = None;
            self.selected_device_info = None;
            self.selected_config = None;
        }
    }

    /// Clamp `device_index` to valid range after data changes.
    pub const fn clamp_device_index(&mut self) {
        let total = self.device_count();
        if total == 0 {
            self.device_index = 0;
        } else if self.device_index >= total {
            self.device_index = total.saturating_sub(1);
        }
    }
}
