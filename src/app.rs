use crate::action::Action;
use crate::mise;
use crate::model::{ConfigFile, InstalledTool, RegistryEntry};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tools,
    Registry,
    Config,
    Doctor,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Tools, Tab::Registry, Tab::Config, Tab::Doctor];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Tools => " Tools",
            Tab::Registry => " Registry",
            Tab::Config => " Config",
            Tab::Doctor => "󰑓 Doctor",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Tools => 0,
            Tab::Registry => 1,
            Tab::Config => 2,
            Tab::Doctor => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Content,
}

#[derive(Debug, Clone)]
pub enum Popup {
    VersionPicker {
        tool: String,
        versions: Vec<String>,
        selected: usize,
    },
    Confirm {
        message: String,
        action_on_confirm: ConfirmAction,
    },
    Progress {
        message: String,
    },
    Help,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Uninstall { tool: String, version: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Loading,
    Loaded,
}

pub struct App {
    pub should_quit: bool,
    pub tab: Tab,
    pub focus: Focus,

    // Data
    pub tools: Vec<InstalledTool>,
    pub registry: Vec<RegistryEntry>,
    pub configs: Vec<ConfigFile>,
    pub doctor_lines: Vec<String>,

    // Load states
    pub tools_state: LoadState,
    pub registry_state: LoadState,
    pub config_state: LoadState,
    pub doctor_state: LoadState,

    // Selection / scroll state
    pub tools_selected: usize,
    pub registry_selected: usize,
    pub config_selected: usize,
    pub doctor_scroll: usize,
    pub sidebar_selected: usize,

    // Search
    pub search_active: bool,
    pub search_query: String,
    pub filtered_registry: Vec<usize>,
    pub filtered_tools: Vec<usize>,
    pub filtered_configs: Vec<usize>,
    pub filtered_doctor: Vec<usize>,

    // Popup
    pub popup: Option<Popup>,

    // Status message (text, TTL ticks remaining)
    pub status_message: Option<(String, usize)>,

    // Spinner
    pub spinner_frame: usize,

    // Action channel for async operations
    pub action_tx: mpsc::UnboundedSender<Action>,
}

impl App {
    pub fn new(action_tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            should_quit: false,
            tab: Tab::Tools,
            focus: Focus::Content,

            tools: Vec::new(),
            registry: Vec::new(),
            configs: Vec::new(),
            doctor_lines: Vec::new(),

            tools_state: LoadState::Loading,
            registry_state: LoadState::Loading,
            config_state: LoadState::Loading,
            doctor_state: LoadState::Loading,

            tools_selected: 0,
            registry_selected: 0,
            config_selected: 0,
            doctor_scroll: 0,
            sidebar_selected: 0,

            search_active: false,
            search_query: String::new(),
            filtered_registry: Vec::new(),
            filtered_tools: Vec::new(),
            filtered_configs: Vec::new(),
            filtered_doctor: Vec::new(),

            popup: None,
            status_message: None,
            spinner_frame: 0,
            action_tx,
        }
    }

    pub fn start_fetch(&self) {
        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            if let Ok(tools) = mise::fetch_tools().await {
                let _ = tx.send(Action::ToolsLoaded(tools));
            }
        });

        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            if let Ok(registry) = mise::fetch_registry().await {
                let _ = tx.send(Action::RegistryLoaded(registry));
            }
        });

        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            if let Ok(configs) = mise::fetch_config().await {
                let _ = tx.send(Action::ConfigLoaded(configs));
            }
        });

        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            if let Ok(lines) = mise::fetch_doctor().await {
                let _ = tx.send(Action::DoctorLoaded(lines));
            }
        });
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => {
                if self.popup.is_some() {
                    self.popup = None;
                } else if self.search_active {
                    self.search_active = false;
                    self.search_query.clear();
                } else {
                    self.should_quit = true;
                }
            }
            Action::MoveUp => self.move_selection(-1),
            Action::MoveDown => self.move_selection(1),
            Action::PageUp => self.move_selection(-10),
            Action::PageDown => self.move_selection(10),
            Action::NextTab => self.cycle_tab(1),
            Action::PrevTab => self.cycle_tab(-1),
            Action::FocusSidebar => self.focus = Focus::Sidebar,
            Action::FocusContent => self.focus = Focus::Content,

            Action::EnterSearch => {
                if self.popup.is_none() {
                    self.search_active = true;
                    self.search_query.clear();
                    self.update_all_filters();
                }
            }
            Action::ExitSearch => {
                self.search_active = false;
            }
            Action::SearchInput(c) => {
                if !self.search_active && self.popup.is_none() {
                    // Auto-activate search on first unbound char
                    self.search_active = true;
                    self.search_query.clear();
                }
                if self.search_active {
                    self.search_query.push(c);
                    self.update_all_filters();
                    self.reset_selection_for_tab();
                }
            }
            Action::SearchBackspace => {
                if self.search_active {
                    self.search_query.pop();
                    self.update_all_filters();
                    self.reset_selection_for_tab();
                }
            }

            Action::ToolsLoaded(tools) => {
                self.tools = tools;
                self.tools_state = LoadState::Loaded;
                self.update_filtered_tools();
            }
            Action::RegistryLoaded(registry) => {
                self.registry = registry;
                self.registry_state = LoadState::Loaded;
                self.update_filtered_registry();
            }
            Action::ConfigLoaded(configs) => {
                self.configs = configs;
                self.config_state = LoadState::Loaded;
                self.update_filtered_configs();
            }
            Action::DoctorLoaded(lines) => {
                self.doctor_lines = lines;
                self.doctor_state = LoadState::Loaded;
                self.update_filtered_doctor();
            }
            Action::VersionsLoaded(versions) => {
                if let Some(Popup::Progress { message }) = &self.popup {
                    let tool = message
                        .strip_prefix("Fetching versions for ")
                        .unwrap_or("")
                        .strip_suffix("...")
                        .unwrap_or("")
                        .to_string();
                    if !versions.is_empty() {
                        self.popup = Some(Popup::VersionPicker {
                            tool,
                            versions,
                            selected: 0,
                        });
                    } else {
                        self.popup = None;
                        self.status_message = Some(("No versions found".to_string(), 20));
                    }
                }
            }

            Action::InstallTool => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Registry {
                    if let Some(entry) = self.selected_registry_entry() {
                        let tool = entry.short.clone();
                        self.popup = Some(Popup::Progress {
                            message: format!("Fetching versions for {tool}..."),
                        });
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            match mise::fetch_versions(&tool).await {
                                Ok(versions) => {
                                    let _ = tx.send(Action::VersionsLoaded(versions));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::OperationFailed(e));
                                }
                            }
                        });
                    }
                }
            }

            Action::UninstallTool => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Tools {
                    if let Some(tool) = self.tools.get(self.tools_selected) {
                        let name = tool.name.clone();
                        let version = tool.version.clone();
                        self.popup = Some(Popup::Confirm {
                            message: format!("Uninstall {name}@{version}?"),
                            action_on_confirm: ConfirmAction::Uninstall {
                                tool: name,
                                version,
                            },
                        });
                    }
                }
            }

            Action::UpdateTool => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Tools {
                    if let Some(tool) = self.tools.get(self.tools_selected) {
                        let name = tool.name.clone();
                        self.popup = Some(Popup::Progress {
                            message: format!("Updating {name}..."),
                        });
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            match mise::update_tool(&name).await {
                                Ok(msg) => {
                                    let _ = tx.send(Action::OperationComplete(msg));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::OperationFailed(e));
                                }
                            }
                        });
                    }
                }
            }

            Action::ConfirmAction => {
                if let Some(popup) = self.popup.take() {
                    match popup {
                        Popup::VersionPicker {
                            tool,
                            versions,
                            selected,
                        } => {
                            if let Some(version) = versions.get(selected) {
                                let version = version.clone();
                                let tool_clone = tool.clone();
                                self.popup = Some(Popup::Progress {
                                    message: format!("Installing {tool}@{version}..."),
                                });
                                let tx = self.action_tx.clone();
                                tokio::spawn(async move {
                                    match mise::install_tool(&tool_clone, &version).await {
                                        Ok(msg) => {
                                            let _ = tx.send(Action::OperationComplete(msg));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::OperationFailed(e));
                                        }
                                    }
                                });
                            }
                        }
                        Popup::Confirm {
                            action_on_confirm, ..
                        } => match action_on_confirm {
                            ConfirmAction::Uninstall { tool, version } => {
                                self.popup = Some(Popup::Progress {
                                    message: format!("Uninstalling {tool}@{version}..."),
                                });
                                let tx = self.action_tx.clone();
                                tokio::spawn(async move {
                                    match mise::uninstall_tool(&tool, &version).await {
                                        Ok(msg) => {
                                            let _ = tx.send(Action::OperationComplete(msg));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::OperationFailed(e));
                                        }
                                    }
                                });
                            }
                        },
                        Popup::Help => {}
                        Popup::Progress { .. } => {
                            self.popup = Some(Popup::Progress {
                                message: "Working...".to_string(),
                            });
                        }
                    }
                }
            }

            Action::CancelPopup => {
                if self.popup.is_some() {
                    self.popup = None;
                } else if self.search_active {
                    self.search_active = false;
                    self.search_query.clear();
                    self.update_all_filters();
                }
            }

            Action::OperationComplete(msg) => {
                self.popup = None;
                self.status_message = Some((msg, 20));
                // Refresh data
                self.start_fetch();
            }

            Action::OperationFailed(msg) => {
                self.popup = None;
                self.status_message = Some((format!("Error: {msg}"), 20));
            }

            Action::ShowHelp => {
                self.popup = Some(Popup::Help);
            }

            Action::Tick => {
                self.spinner_frame = (self.spinner_frame + 1) % 10;
                if let Some((_, ttl)) = &mut self.status_message {
                    if *ttl == 0 {
                        self.status_message = None;
                    } else {
                        *ttl -= 1;
                    }
                }
            }

            Action::Render | Action::None => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if let Some(popup) = &mut self.popup {
            match popup {
                Popup::VersionPicker {
                    selected, versions, ..
                } => {
                    Self::adjust_selection(selected, delta, versions.len());
                    return;
                }
                _ => return,
            }
        }

        if self.focus == Focus::Sidebar {
            let len = Tab::ALL.len();
            Self::adjust_selection(&mut self.sidebar_selected, delta, len);
            self.tab = Tab::ALL[self.sidebar_selected];
            return;
        }

        match self.tab {
            Tab::Tools => {
                let len = self.filtered_tools.len();
                Self::adjust_selection(&mut self.tools_selected, delta, len);
            }
            Tab::Registry => {
                let len = self.filtered_registry.len();
                Self::adjust_selection(&mut self.registry_selected, delta, len);
            }
            Tab::Config => {
                let len = self.filtered_configs.len();
                Self::adjust_selection(&mut self.config_selected, delta, len);
            }
            Tab::Doctor => {
                let total = self.filtered_doctor.len();
                Self::adjust_scroll(&mut self.doctor_scroll, delta, total);
            }
        }
    }

    fn adjust_selection(selected: &mut usize, delta: i32, len: usize) {
        if len == 0 {
            *selected = 0;
            return;
        }
        let new_val = (*selected as i32 + delta).clamp(0, (len as i32) - 1) as usize;
        *selected = new_val;
    }

    fn adjust_scroll(scroll: &mut usize, delta: i32, total: usize) {
        if total == 0 {
            *scroll = 0;
            return;
        }
        let new_val = (*scroll as i32 + delta).clamp(0, (total as i32) - 1) as usize;
        *scroll = new_val;
    }

    fn cycle_tab(&mut self, delta: i32) {
        let idx = self.tab.index() as i32;
        let len = Tab::ALL.len() as i32;
        let new_idx = (idx + delta).rem_euclid(len) as usize;
        self.tab = Tab::ALL[new_idx];
        self.sidebar_selected = new_idx;
    }

    fn update_all_filters(&mut self) {
        self.update_filtered_registry();
        self.update_filtered_tools();
        self.update_filtered_configs();
        self.update_filtered_doctor();
    }

    fn reset_selection_for_tab(&mut self) {
        match self.tab {
            Tab::Tools => self.tools_selected = 0,
            Tab::Registry => self.registry_selected = 0,
            Tab::Config => self.config_selected = 0,
            Tab::Doctor => self.doctor_scroll = 0,
        }
    }

    fn update_filtered_registry(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_registry = (0..self.registry.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_registry = self
                .registry
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    entry.short.to_lowercase().contains(&q)
                        || entry
                            .description
                            .as_ref()
                            .is_some_and(|d| d.to_lowercase().contains(&q))
                        || entry.aliases.iter().any(|a| a.to_lowercase().contains(&q))
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn update_filtered_tools(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_tools = (0..self.tools.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_tools = self
                .tools
                .iter()
                .enumerate()
                .filter(|(_, tool)| {
                    tool.name.to_lowercase().contains(&q)
                        || tool.version.to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn update_filtered_configs(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_configs = (0..self.configs.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_configs = self
                .configs
                .iter()
                .enumerate()
                .filter(|(_, cfg)| {
                    cfg.path.to_lowercase().contains(&q)
                        || cfg.tools.iter().any(|t| t.to_lowercase().contains(&q))
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn update_filtered_doctor(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_doctor = (0..self.doctor_lines.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_doctor = self
                .doctor_lines
                .iter()
                .enumerate()
                .filter(|(_, line)| line.to_lowercase().contains(&q))
                .map(|(i, _)| i)
                .collect();
        }
    }

    pub fn selected_registry_entry(&self) -> Option<&RegistryEntry> {
        self.filtered_registry
            .get(self.registry_selected)
            .and_then(|&i| self.registry.get(i))
    }

    pub fn visible_registry_entries(&self) -> Vec<&RegistryEntry> {
        self.filtered_registry
            .iter()
            .filter_map(|&i| self.registry.get(i))
            .collect()
    }

    pub fn visible_tools(&self) -> Vec<&InstalledTool> {
        self.filtered_tools
            .iter()
            .filter_map(|&i| self.tools.get(i))
            .collect()
    }

    pub fn visible_configs(&self) -> Vec<&ConfigFile> {
        self.filtered_configs
            .iter()
            .filter_map(|&i| self.configs.get(i))
            .collect()
    }

    pub fn visible_doctor_lines(&self) -> Vec<&String> {
        self.filtered_doctor
            .iter()
            .filter_map(|&i| self.doctor_lines.get(i))
            .collect()
    }

    pub fn spinner_char(&self) -> char {
        const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER[self.spinner_frame]
    }

    pub fn outdated_count(&self) -> usize {
        self.tools
            .iter()
            .filter(|t| {
                !t.requested_version.is_empty()
                    && t.requested_version == "latest"
            })
            .count()
    }
}
