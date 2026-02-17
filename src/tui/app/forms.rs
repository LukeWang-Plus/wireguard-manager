//! Form open/submit logic for all modal dialogs (init, add, edit, network config).

use crate::manager;
use crate::tui::input::TextInput;

use super::{App, FormFocus, Mode};

impl App {
    pub(super) fn open_init_form(&mut self) {
        self.form_fields = vec![
            TextInput::with_value("10.0.0.0/24"),
            TextInput::with_value("[1,10]"),
        ];
        self.form_labels = vec!["Subnet", "Server Range"];
        self.form_focus = FormFocus::Field(0);
        self.mode = Mode::InitNetwork;
    }

    pub(super) fn open_edit_network_form(&mut self) {
        let (subnet, range) = self.network_info.as_ref().map_or_else(
            || (String::new(), String::new()),
            |info| {
                (
                    info.subnet.clone(),
                    format!(
                        "[{},{}]",
                        info.server_range_offsets.0, info.server_range_offsets.1
                    ),
                )
            },
        );
        self.form_fields = vec![
            TextInput::with_value(&subnet),
            TextInput::with_value(&range),
        ];
        self.form_labels = vec!["Subnet", "Server Range"];
        self.form_focus = FormFocus::Field(0);
        self.mode = Mode::EditNetworkConfig;
    }

    pub(super) fn submit_init(&mut self) {
        let Some(subnet_field) = self.form_fields.first() else {
            return;
        };
        let Some(range_field) = self.form_fields.get(1) else {
            return;
        };
        let subnet = subnet_field.value().to_string();
        let range_raw = range_field.value().to_string();
        let Ok(parsed) = manager::parse_server_range(&range_raw) else {
            self.set_status(format!(
                "Invalid server range '{range_raw}'. Use [START,END]"
            ));
            return;
        };
        match self
            .manager
            .init_network(&subnet, parsed.0, parsed.1, self.force_init)
        {
            Ok(result) => {
                self.handle_operation_result(&result);
                self.mode = Mode::Normal;
                self.force_init = false;
                self.refresh_data();
            }
            Err(e) => self.set_status(format!("Error: {e}")),
        }
    }

    pub(super) fn submit_edit_network(&mut self) {
        let Some(subnet_field) = self.form_fields.first() else {
            return;
        };
        let Some(range_field) = self.form_fields.get(1) else {
            return;
        };
        let subnet = subnet_field.value().to_string();
        let range_raw = range_field.value().to_string();
        let (ss, se) = if let Ok((s, e)) = manager::parse_server_range(&range_raw) {
            (Some(s), Some(e))
        } else {
            self.set_status(format!(
                "Invalid server range '{range_raw}'. Use [START,END]"
            ));
            return;
        };
        match self.manager.update_network_config(Some(&subnet), ss, se) {
            Ok(result) => {
                self.handle_operation_result(&result);
                self.mode = Mode::Normal;
                self.refresh_data();
            }
            Err(e) => self.set_status(format!("Error: {e}")),
        }
    }

    pub(super) fn open_add_server_form(&mut self) {
        if self.network_info.is_none() {
            self.set_status("Network not initialized. Press CTRL+I to initialize first.");
            return;
        }
        self.form_fields = vec![
            TextInput::new(),
            TextInput::new(),
            TextInput::with_value("51820"),
        ];
        self.form_labels = vec!["Name", "Public IP", "Port"];
        self.form_focus = FormFocus::Field(0);
        self.mode = Mode::AddServer;
    }

    pub(super) fn open_edit_server_form(&mut self) {
        // Find the server corresponding to device_index
        let server_idx = self.device_index;
        if let Some(server) = self.servers.get(server_idx) {
            let name = server.name.clone();
            self.form_fields = vec![
                TextInput::with_value(&server.public_ip),
                TextInput::with_value(&server.listen_port.to_string()),
            ];
            self.form_labels = vec!["Public IP", "Port"];
            self.form_focus = FormFocus::Field(0);
            self.mode = Mode::EditServer { name };
        }
    }

    pub(super) fn open_delete_server(&mut self) {
        let server_idx = self.device_index;
        if let Some(server) = self.servers.get(server_idx) {
            self.mode = Mode::ConfirmDelete {
                name: server.name.clone(),
                device_type: "server".to_string(),
            };
        }
    }

    pub(super) fn submit_add_server(&mut self) {
        let Some(name_field) = self.form_fields.first() else {
            return;
        };
        let Some(ip_field) = self.form_fields.get(1) else {
            return;
        };
        let Some(port_field) = self.form_fields.get(2) else {
            return;
        };
        let name = name_field.value().to_string();
        let public_ip = ip_field.value().to_string();
        let port_str = port_field.value().to_string();
        let port: u16 = if let Ok(p) = port_str.parse() {
            p
        } else {
            self.set_status(format!("Invalid port: {port_str}"));
            return;
        };
        match self.manager.add_server(&name, &public_ip, port) {
            Ok(result) => {
                self.handle_operation_result(&result);
                self.mode = Mode::Normal;
                self.refresh_data();
            }
            Err(e) => self.set_status(format!("Error: {e}")),
        }
    }

    pub(super) fn submit_edit_server(&mut self, original_name: &str) {
        let Some(ip_field) = self.form_fields.first() else {
            return;
        };
        let Some(port_field) = self.form_fields.get(1) else {
            return;
        };
        let public_ip = ip_field.value().to_string();
        let port_str = port_field.value().to_string();
        let port: u16 = if let Ok(p) = port_str.parse() {
            p
        } else {
            self.set_status(format!("Invalid port: {port_str}"));
            return;
        };
        match self
            .manager
            .edit_server(original_name, Some(&public_ip), Some(port))
        {
            Ok(result) => {
                self.handle_operation_result(&result);
                self.mode = Mode::Normal;
                self.refresh_data();
            }
            Err(e) => self.set_status(format!("Error: {e}")),
        }
    }

    pub(super) fn open_add_client_form(&mut self) {
        if self.network_info.is_none() {
            self.set_status("Network not initialized. Press CTRL+I to initialize first.");
            return;
        }
        self.form_fields = vec![TextInput::new()];
        self.form_labels = vec!["Name"];
        self.form_focus = FormFocus::Field(0);
        self.mode = Mode::AddClient;
    }

    pub(super) fn open_edit_client_form(&mut self) {
        // Find the client corresponding to device_index
        let client_idx = self.device_index.saturating_sub(self.servers.len());
        if let Some(client) = self.clients.get(client_idx) {
            let name = client.name.clone();
            self.form_fields = vec![TextInput::with_value(&client.name)];
            self.form_labels = vec!["New Name"];
            self.form_focus = FormFocus::Field(0);
            self.mode = Mode::EditClient { name };
        }
    }

    pub(super) fn open_delete_client(&mut self) {
        let client_idx = self.device_index.saturating_sub(self.servers.len());
        if let Some(client) = self.clients.get(client_idx) {
            self.mode = Mode::ConfirmDelete {
                name: client.name.clone(),
                device_type: "client".to_string(),
            };
        }
    }

    pub(super) fn submit_add_client(&mut self) {
        let Some(name_field) = self.form_fields.first() else {
            return;
        };
        let name = name_field.value().to_string();
        match self.manager.add_client(&name) {
            Ok(result) => {
                self.handle_operation_result(&result);
                self.mode = Mode::Normal;
                self.refresh_data();
            }
            Err(e) => self.set_status(format!("Error: {e}")),
        }
    }

    pub(super) fn submit_edit_client(&mut self, original_name: &str) {
        let Some(name_field) = self.form_fields.first() else {
            return;
        };
        let new_name = name_field.value().to_string();
        match self.manager.edit_client(original_name, Some(&new_name)) {
            Ok(result) => {
                self.handle_operation_result(&result);
                self.mode = Mode::Normal;
                self.refresh_data();
            }
            Err(e) => self.set_status(format!("Error: {e}")),
        }
    }

    /// Open edit form for whatever device is currently selected.
    pub(super) fn open_edit_selected(&mut self) {
        if self.device_count() == 0 {
            return;
        }
        if self.is_server_at(self.device_index) {
            self.open_edit_server_form();
        } else {
            self.open_edit_client_form();
        }
    }

    /// Open delete confirmation for whatever device is currently selected.
    pub(super) fn open_delete_selected(&mut self) {
        if self.device_count() == 0 {
            return;
        }
        if self.is_server_at(self.device_index) {
            self.open_delete_server();
        } else {
            self.open_delete_client();
        }
    }
}
