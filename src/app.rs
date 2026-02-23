use crate::action::Action;
use crate::mise;
use crate::model::{
    ConfigFile, EnvVar, InstalledTool, MiseSetting, MiseTask, OutdatedTool, RegistryEntry,
};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tools,
    Outdated,
    Registry,
    Tasks,
    Environment,
    Settings,
    Config,
    Doctor,
}

impl Tab {
    pub const ALL: [Tab; 8] = [
        Tab::Tools,
        Tab::Outdated,
        Tab::Registry,
        Tab::Tasks,
        Tab::Environment,
        Tab::Settings,
        Tab::Config,
        Tab::Doctor,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Tools => " Tools",
            Tab::Outdated => " Outdated",
            Tab::Registry => " Registry",
            Tab::Tasks => " Tasks",
            Tab::Environment => " Env",
            Tab::Settings => " Settings",
            Tab::Config => " Config",
            Tab::Doctor => "󰑓 Doctor",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Tools => 0,
            Tab::Outdated => 1,
            Tab::Registry => 2,
            Tab::Tasks => 3,
            Tab::Environment => 4,
            Tab::Settings => 5,
            Tab::Config => 6,
            Tab::Doctor => 7,
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
        use_global: bool,
        search_query: String,
        filtered_versions: Vec<usize>,
    },
    Confirm {
        message: String,
        action_on_confirm: ConfirmAction,
    },
    Progress {
        message: String,
    },
    ToolDetail {
        tool_name: String,
        info: String,
        scroll: usize,
    },
    Help,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Uninstall { tool: String, version: String },
    Prune,
    TrustConfig { path: String },
    RunTask { task: String },
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
    pub outdated: Vec<OutdatedTool>,
    pub tasks: Vec<MiseTask>,
    pub env_vars: Vec<EnvVar>,
    pub settings: Vec<MiseSetting>,

    // Cross-reference
    pub outdated_map: HashMap<String, OutdatedTool>,

    // Load states
    pub tools_state: LoadState,
    pub registry_state: LoadState,
    pub config_state: LoadState,
    pub doctor_state: LoadState,
    pub outdated_state: LoadState,
    pub tasks_state: LoadState,
    pub env_state: LoadState,
    pub settings_state: LoadState,

    // Selection / scroll state
    pub tools_selected: usize,
    pub registry_selected: usize,
    pub config_selected: usize,
    pub doctor_scroll: usize,
    pub sidebar_selected: usize,
    pub outdated_selected: usize,
    pub tasks_selected: usize,
    pub env_selected: usize,
    pub settings_selected: usize,

    // Search
    pub search_active: bool,
    pub search_query: String,
    pub filtered_registry: Vec<usize>,
    pub filtered_tools: Vec<usize>,
    pub filtered_configs: Vec<usize>,
    pub filtered_doctor: Vec<usize>,
    pub filtered_outdated: Vec<usize>,
    pub filtered_tasks: Vec<usize>,
    pub filtered_env: Vec<usize>,
    pub filtered_settings: Vec<usize>,

    // Sorting
    pub sort_column: usize,
    pub sort_ascending: bool,

    // Popup
    pub popup: Option<Popup>,

    // Use global flag for version picker flow
    pub pending_use_global: bool,

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
            outdated: Vec::new(),
            tasks: Vec::new(),
            env_vars: Vec::new(),
            settings: Vec::new(),

            outdated_map: HashMap::new(),

            tools_state: LoadState::Loading,
            registry_state: LoadState::Loading,
            config_state: LoadState::Loading,
            doctor_state: LoadState::Loading,
            outdated_state: LoadState::Loading,
            tasks_state: LoadState::Loading,
            env_state: LoadState::Loading,
            settings_state: LoadState::Loading,

            tools_selected: 0,
            registry_selected: 0,
            config_selected: 0,
            doctor_scroll: 0,
            sidebar_selected: 0,
            outdated_selected: 0,
            tasks_selected: 0,
            env_selected: 0,
            settings_selected: 0,

            search_active: false,
            search_query: String::new(),
            filtered_registry: Vec::new(),
            filtered_tools: Vec::new(),
            filtered_configs: Vec::new(),
            filtered_doctor: Vec::new(),
            filtered_outdated: Vec::new(),
            filtered_tasks: Vec::new(),
            filtered_env: Vec::new(),
            filtered_settings: Vec::new(),

            sort_column: 0,
            sort_ascending: true,

            popup: None,
            pending_use_global: false,
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

        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            if let Ok(outdated) = mise::fetch_outdated().await {
                let _ = tx.send(Action::OutdatedLoaded(outdated));
            }
        });

        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            if let Ok(tasks) = mise::fetch_tasks().await {
                let _ = tx.send(Action::TasksLoaded(tasks));
            }
        });

        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            if let Ok(env) = mise::fetch_env().await {
                let _ = tx.send(Action::EnvLoaded(env));
            }
        });

        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            if let Ok(settings) = mise::fetch_settings().await {
                let _ = tx.send(Action::SettingsLoaded(settings));
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
            Action::OutdatedLoaded(outdated) => {
                self.outdated_map = outdated
                    .iter()
                    .map(|o| (o.name.clone(), o.clone()))
                    .collect();
                self.outdated = outdated;
                self.outdated_state = LoadState::Loaded;
                self.update_filtered_outdated();
            }
            Action::TasksLoaded(tasks) => {
                self.tasks = tasks;
                self.tasks_state = LoadState::Loaded;
                self.update_filtered_tasks();
            }
            Action::EnvLoaded(env) => {
                self.env_vars = env;
                self.env_state = LoadState::Loaded;
                self.update_filtered_env();
            }
            Action::SettingsLoaded(settings) => {
                self.settings = settings;
                self.settings_state = LoadState::Loaded;
                self.update_filtered_settings();
            }
            Action::PruneLoaded(candidates) => {
                if candidates.is_empty() {
                    self.popup = None;
                    self.status_message =
                        Some(("No unused tool versions to prune".to_string(), 20));
                } else {
                    let count = candidates.len();
                    let names: Vec<String> = candidates
                        .iter()
                        .take(5)
                        .map(|c| {
                            if c.version.is_empty() {
                                c.tool.clone()
                            } else {
                                format!("{}@{}", c.tool, c.version)
                            }
                        })
                        .collect();
                    let mut msg = format!("Prune {count} versions? ({}", names.join(", "));
                    if count > 5 {
                        msg.push_str(", ...");
                    }
                    msg.push(')');
                    self.popup = Some(Popup::Confirm {
                        message: msg,
                        action_on_confirm: ConfirmAction::Prune,
                    });
                }
            }
            Action::ToolInfoLoaded(info) => {
                if let Some(Popup::Progress { message }) = &self.popup {
                    let tool_name = message
                        .strip_prefix("Fetching info for ")
                        .unwrap_or("")
                        .strip_suffix("...")
                        .unwrap_or("")
                        .to_string();
                    // Pretty-print the JSON
                    let pretty = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&info)
                    {
                        serde_json::to_string_pretty(&val).unwrap_or(info)
                    } else {
                        info
                    };
                    self.popup = Some(Popup::ToolDetail {
                        tool_name,
                        info: pretty,
                        scroll: 0,
                    });
                }
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
                        let use_global = self.pending_use_global;
                        let len = versions.len();
                        self.popup = Some(Popup::VersionPicker {
                            tool,
                            versions,
                            selected: 0,
                            use_global,
                            search_query: String::new(),
                            filtered_versions: (0..len).collect(),
                        });
                        self.pending_use_global = false;
                    } else {
                        self.popup = None;
                        self.pending_use_global = false;
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
                        self.pending_use_global = false;
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

            Action::UseTool => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Registry {
                    if let Some(entry) = self.selected_registry_entry() {
                        let tool = entry.short.clone();
                        self.pending_use_global = true;
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
                } else if self.tab == Tab::Outdated {
                    // U on Outdated tab upgrades all
                    self.handle_action(Action::UpgradeAll);
                }
            }

            Action::UninstallTool => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Tools {
                    let tools = self.visible_tools_vec();
                    if let Some(tool) = tools.get(self.tools_selected) {
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
                    let tools = self.visible_tools_vec();
                    if let Some(tool) = tools.get(self.tools_selected) {
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
                } else if self.tab == Tab::Outdated {
                    // u on Outdated tab upgrades selected tool
                    let outdated = self.visible_outdated();
                    if let Some(tool) = outdated.get(self.outdated_selected) {
                        let name = tool.name.clone();
                        self.popup = Some(Popup::Progress {
                            message: format!("Upgrading {name}..."),
                        });
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            match mise::upgrade_tool(&name).await {
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

            Action::UpgradeAll => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Outdated {
                    self.popup = Some(Popup::Progress {
                        message: "Upgrading all tools...".to_string(),
                    });
                    let tx = self.action_tx.clone();
                    tokio::spawn(async move {
                        match mise::upgrade_all().await {
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

            Action::RunTask => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Tasks {
                    let tasks = self.visible_tasks();
                    if let Some(task) = tasks.get(self.tasks_selected) {
                        let name = task.name.clone();
                        self.popup = Some(Popup::Confirm {
                            message: format!("Run task '{name}'?"),
                            action_on_confirm: ConfirmAction::RunTask {
                                task: name,
                            },
                        });
                    }
                }
            }

            Action::PruneTool => {
                if self.popup.is_some() {
                    return;
                }
                self.popup = Some(Popup::Progress {
                    message: "Checking for unused versions...".to_string(),
                });
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    match mise::prune_dry_run().await {
                        Ok(candidates) => {
                            let _ = tx.send(Action::PruneLoaded(candidates));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::OperationFailed(e));
                        }
                    }
                });
            }

            Action::TrustConfig => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Config {
                    let configs = self.visible_configs_vec();
                    if let Some(cfg) = configs.get(self.config_selected) {
                        let path = cfg.path.clone();
                        self.popup = Some(Popup::Confirm {
                            message: format!("Trust config: {path}?"),
                            action_on_confirm: ConfirmAction::TrustConfig { path },
                        });
                    }
                }
            }

            Action::ShowToolDetail => {
                if self.popup.is_some() {
                    return;
                }
                if self.tab == Tab::Tools {
                    let tools = self.visible_tools_vec();
                    if let Some(tool) = tools.get(self.tools_selected) {
                        let name = tool.name.clone();
                        self.popup = Some(Popup::Progress {
                            message: format!("Fetching info for {name}..."),
                        });
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            match mise::fetch_tool_info(&name).await {
                                Ok(info) => {
                                    let _ = tx.send(Action::ToolInfoLoaded(info));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::OperationFailed(e));
                                }
                            }
                        });
                    }
                } else if self.tab == Tab::Tasks {
                    // Enter on Tasks tab runs the selected task
                    self.handle_action(Action::RunTask);
                }
            }

            Action::CycleSortOrder => {
                if self.popup.is_some() {
                    return;
                }
                if self.sort_ascending {
                    self.sort_ascending = false;
                } else {
                    self.sort_ascending = true;
                    // Advance column
                    let max_col = match self.tab {
                        Tab::Tools => 3,
                        Tab::Registry => 1,
                        Tab::Outdated => 3,
                        Tab::Tasks => 2,
                        Tab::Environment => 3,
                        Tab::Settings => 2,
                        _ => 0,
                    };
                    if max_col > 0 {
                        self.sort_column = (self.sort_column + 1) % (max_col + 1);
                    }
                }
                self.apply_sort();
            }

            Action::Refresh => {
                self.tools_state = LoadState::Loading;
                self.registry_state = LoadState::Loading;
                self.config_state = LoadState::Loading;
                self.doctor_state = LoadState::Loading;
                self.outdated_state = LoadState::Loading;
                self.tasks_state = LoadState::Loading;
                self.env_state = LoadState::Loading;
                self.settings_state = LoadState::Loading;
                self.status_message = Some(("Refreshing...".to_string(), 10));
                self.start_fetch();
            }

            Action::MouseClick { x, y } => {
                // Sidebar width is 16
                if x < 16 {
                    // Map y to tab index (header takes 3 rows, border takes 1)
                    let tab_y = y.saturating_sub(4) as usize;
                    if tab_y < Tab::ALL.len() {
                        self.tab = Tab::ALL[tab_y];
                        self.sidebar_selected = tab_y;
                        self.sort_column = 0;
                        self.sort_ascending = true;
                    }
                } else {
                    self.focus = Focus::Content;
                }
            }

            Action::PopupSearchInput(c) => {
                if let Some(Popup::VersionPicker {
                    ref mut search_query,
                    ref mut filtered_versions,
                    ref mut selected,
                    ref versions,
                    ..
                }) = self.popup
                {
                    search_query.push(c);
                    let q = search_query.to_lowercase();
                    *filtered_versions = versions
                        .iter()
                        .enumerate()
                        .filter(|(_, v)| v.to_lowercase().contains(&q))
                        .map(|(i, _)| i)
                        .collect();
                    *selected = 0;
                }
            }

            Action::PopupSearchBackspace => {
                if let Some(Popup::VersionPicker {
                    ref mut search_query,
                    ref mut filtered_versions,
                    ref mut selected,
                    ref versions,
                    ..
                }) = self.popup
                {
                    search_query.pop();
                    if search_query.is_empty() {
                        *filtered_versions = (0..versions.len()).collect();
                    } else {
                        let q = search_query.to_lowercase();
                        *filtered_versions = versions
                            .iter()
                            .enumerate()
                            .filter(|(_, v)| v.to_lowercase().contains(&q))
                            .map(|(i, _)| i)
                            .collect();
                    }
                    *selected = 0;
                }
            }

            Action::Confirm => {
                if let Some(popup) = self.popup.take() {
                    match popup {
                        Popup::VersionPicker {
                            tool,
                            versions,
                            selected,
                            use_global,
                            filtered_versions,
                            ..
                        } => {
                            // Use filtered index to get actual version
                            let actual_idx = filtered_versions.get(selected).copied();
                            if let Some(idx) = actual_idx {
                                if let Some(version) = versions.get(idx) {
                                    let version = version.clone();
                                    let tool_clone = tool.clone();
                                    if use_global {
                                        self.popup = Some(Popup::Progress {
                                            message: format!(
                                                "Setting {tool}@{version} globally..."
                                            ),
                                        });
                                        let tx = self.action_tx.clone();
                                        tokio::spawn(async move {
                                            match mise::use_tool(&tool_clone, &version).await {
                                                Ok(msg) => {
                                                    let _ =
                                                        tx.send(Action::OperationComplete(msg));
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(Action::OperationFailed(e));
                                                }
                                            }
                                        });
                                    } else {
                                        self.popup = Some(Popup::Progress {
                                            message: format!("Installing {tool}@{version}..."),
                                        });
                                        let tx = self.action_tx.clone();
                                        tokio::spawn(async move {
                                            match mise::install_tool(&tool_clone, &version).await {
                                                Ok(msg) => {
                                                    let _ =
                                                        tx.send(Action::OperationComplete(msg));
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(Action::OperationFailed(e));
                                                }
                                            }
                                        });
                                    }
                                }
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
                            ConfirmAction::Prune => {
                                self.popup = Some(Popup::Progress {
                                    message: "Pruning unused versions...".to_string(),
                                });
                                let tx = self.action_tx.clone();
                                tokio::spawn(async move {
                                    match mise::prune().await {
                                        Ok(msg) => {
                                            let _ = tx.send(Action::OperationComplete(msg));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::OperationFailed(e));
                                        }
                                    }
                                });
                            }
                            ConfirmAction::TrustConfig { path } => {
                                self.popup = Some(Popup::Progress {
                                    message: format!("Trusting {path}..."),
                                });
                                let tx = self.action_tx.clone();
                                tokio::spawn(async move {
                                    match mise::trust_config(&path).await {
                                        Ok(msg) => {
                                            let _ = tx.send(Action::OperationComplete(msg));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::OperationFailed(e));
                                        }
                                    }
                                });
                            }
                            ConfirmAction::RunTask { task } => {
                                self.popup = Some(Popup::Progress {
                                    message: format!("Running task '{task}'..."),
                                });
                                let tx = self.action_tx.clone();
                                tokio::spawn(async move {
                                    match mise::run_task(&task).await {
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
                        Popup::ToolDetail { .. } => {}
                        Popup::Progress { .. } => {
                            self.popup = Some(Popup::Progress {
                                message: "Working...".to_string(),
                            });
                        }
                    }
                } else {
                    // No popup open — dispatch Enter contextually by tab
                    match self.tab {
                        Tab::Tools => self.handle_action(Action::ShowToolDetail),
                        Tab::Tasks => self.handle_action(Action::RunTask),
                        _ => {}
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
                    selected,
                    filtered_versions,
                    ..
                } => {
                    Self::adjust_selection(selected, delta, filtered_versions.len());
                    return;
                }
                Popup::ToolDetail { scroll, info, .. } => {
                    let lines = info.lines().count();
                    Self::adjust_scroll(scroll, delta, lines);
                    return;
                }
                _ => return,
            }
        }

        if self.focus == Focus::Sidebar {
            let len = Tab::ALL.len();
            Self::adjust_selection(&mut self.sidebar_selected, delta, len);
            self.tab = Tab::ALL[self.sidebar_selected];
            self.sort_column = 0;
            self.sort_ascending = true;
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
            Tab::Outdated => {
                let len = self.filtered_outdated.len();
                Self::adjust_selection(&mut self.outdated_selected, delta, len);
            }
            Tab::Tasks => {
                let len = self.filtered_tasks.len();
                Self::adjust_selection(&mut self.tasks_selected, delta, len);
            }
            Tab::Environment => {
                let len = self.filtered_env.len();
                Self::adjust_selection(&mut self.env_selected, delta, len);
            }
            Tab::Settings => {
                let len = self.filtered_settings.len();
                Self::adjust_selection(&mut self.settings_selected, delta, len);
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
        self.sort_column = 0;
        self.sort_ascending = true;
    }

    fn update_all_filters(&mut self) {
        self.update_filtered_registry();
        self.update_filtered_tools();
        self.update_filtered_configs();
        self.update_filtered_doctor();
        self.update_filtered_outdated();
        self.update_filtered_tasks();
        self.update_filtered_env();
        self.update_filtered_settings();
    }

    fn reset_selection_for_tab(&mut self) {
        match self.tab {
            Tab::Tools => self.tools_selected = 0,
            Tab::Registry => self.registry_selected = 0,
            Tab::Config => self.config_selected = 0,
            Tab::Doctor => self.doctor_scroll = 0,
            Tab::Outdated => self.outdated_selected = 0,
            Tab::Tasks => self.tasks_selected = 0,
            Tab::Environment => self.env_selected = 0,
            Tab::Settings => self.settings_selected = 0,
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

    fn update_filtered_outdated(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_outdated = (0..self.outdated.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_outdated = self
                .outdated
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.name.to_lowercase().contains(&q)
                        || o.current.to_lowercase().contains(&q)
                        || o.latest.to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn update_filtered_tasks(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_tasks = (0..self.tasks.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_tasks = self
                .tasks
                .iter()
                .enumerate()
                .filter(|(_, t)| {
                    t.name.to_lowercase().contains(&q)
                        || t.description.to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn update_filtered_env(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_env = (0..self.env_vars.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_env = self
                .env_vars
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.name.to_lowercase().contains(&q)
                        || e.value.to_lowercase().contains(&q)
                        || e.source.to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn update_filtered_settings(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_settings = (0..self.settings.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_settings = self
                .settings
                .iter()
                .enumerate()
                .filter(|(_, s)| {
                    s.key.to_lowercase().contains(&q) || s.value.to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn apply_sort(&mut self) {
        let asc = self.sort_ascending;
        let col = self.sort_column;
        match self.tab {
            Tab::Tools => {
                self.filtered_tools.sort_by(|&a, &b| {
                    let ta = &self.tools[a];
                    let tb = &self.tools[b];
                    let cmp = match col {
                        0 => ta.name.to_lowercase().cmp(&tb.name.to_lowercase()),
                        1 => ta.version.cmp(&tb.version),
                        2 => ta.active.cmp(&tb.active),
                        _ => ta.source.to_lowercase().cmp(&tb.source.to_lowercase()),
                    };
                    if asc { cmp } else { cmp.reverse() }
                });
            }
            Tab::Registry => {
                self.filtered_registry.sort_by(|&a, &b| {
                    let ra = &self.registry[a];
                    let rb = &self.registry[b];
                    let cmp = match col {
                        0 => ra.short.to_lowercase().cmp(&rb.short.to_lowercase()),
                        _ => {
                            let da = ra.description.as_deref().unwrap_or("");
                            let db = rb.description.as_deref().unwrap_or("");
                            da.to_lowercase().cmp(&db.to_lowercase())
                        }
                    };
                    if asc { cmp } else { cmp.reverse() }
                });
            }
            Tab::Outdated => {
                self.filtered_outdated.sort_by(|&a, &b| {
                    let oa = &self.outdated[a];
                    let ob = &self.outdated[b];
                    let cmp = match col {
                        0 => oa.name.to_lowercase().cmp(&ob.name.to_lowercase()),
                        1 => oa.current.cmp(&ob.current),
                        2 => oa.latest.cmp(&ob.latest),
                        _ => oa.requested.cmp(&ob.requested),
                    };
                    if asc { cmp } else { cmp.reverse() }
                });
            }
            Tab::Tasks => {
                self.filtered_tasks.sort_by(|&a, &b| {
                    let ta = &self.tasks[a];
                    let tb = &self.tasks[b];
                    let cmp = match col {
                        0 => ta.name.to_lowercase().cmp(&tb.name.to_lowercase()),
                        1 => ta.description.to_lowercase().cmp(&tb.description.to_lowercase()),
                        _ => ta.source.to_lowercase().cmp(&tb.source.to_lowercase()),
                    };
                    if asc { cmp } else { cmp.reverse() }
                });
            }
            Tab::Environment => {
                self.filtered_env.sort_by(|&a, &b| {
                    let ea = &self.env_vars[a];
                    let eb = &self.env_vars[b];
                    let cmp = match col {
                        0 => ea.name.to_lowercase().cmp(&eb.name.to_lowercase()),
                        1 => ea.value.to_lowercase().cmp(&eb.value.to_lowercase()),
                        2 => ea.source.to_lowercase().cmp(&eb.source.to_lowercase()),
                        _ => ea.tool.to_lowercase().cmp(&eb.tool.to_lowercase()),
                    };
                    if asc { cmp } else { cmp.reverse() }
                });
            }
            Tab::Settings => {
                self.filtered_settings.sort_by(|&a, &b| {
                    let sa = &self.settings[a];
                    let sb = &self.settings[b];
                    let cmp = match col {
                        0 => sa.key.to_lowercase().cmp(&sb.key.to_lowercase()),
                        1 => sa.value.to_lowercase().cmp(&sb.value.to_lowercase()),
                        _ => sa.value_type.to_lowercase().cmp(&sb.value_type.to_lowercase()),
                    };
                    if asc { cmp } else { cmp.reverse() }
                });
            }
            _ => {}
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

    fn visible_tools_vec(&self) -> Vec<InstalledTool> {
        self.filtered_tools
            .iter()
            .filter_map(|&i| self.tools.get(i).cloned())
            .collect()
    }

    pub fn visible_configs(&self) -> Vec<&ConfigFile> {
        self.filtered_configs
            .iter()
            .filter_map(|&i| self.configs.get(i))
            .collect()
    }

    fn visible_configs_vec(&self) -> Vec<ConfigFile> {
        self.filtered_configs
            .iter()
            .filter_map(|&i| self.configs.get(i).cloned())
            .collect()
    }

    pub fn visible_doctor_lines(&self) -> Vec<&String> {
        self.filtered_doctor
            .iter()
            .filter_map(|&i| self.doctor_lines.get(i))
            .collect()
    }

    pub fn visible_outdated(&self) -> Vec<&OutdatedTool> {
        self.filtered_outdated
            .iter()
            .filter_map(|&i| self.outdated.get(i))
            .collect()
    }

    pub fn visible_tasks(&self) -> Vec<&MiseTask> {
        self.filtered_tasks
            .iter()
            .filter_map(|&i| self.tasks.get(i))
            .collect()
    }

    pub fn visible_env(&self) -> Vec<&EnvVar> {
        self.filtered_env
            .iter()
            .filter_map(|&i| self.env_vars.get(i))
            .collect()
    }

    pub fn visible_settings(&self) -> Vec<&MiseSetting> {
        self.filtered_settings
            .iter()
            .filter_map(|&i| self.settings.get(i))
            .collect()
    }

    pub fn spinner_char(&self) -> char {
        const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER[self.spinner_frame]
    }

    pub fn outdated_count(&self) -> usize {
        self.outdated.len()
    }

    pub fn sort_indicator(&self, col: usize) -> &str {
        if self.sort_column == col {
            if self.sort_ascending {
                " ▲"
            } else {
                " ▼"
            }
        } else {
            ""
        }
    }
}
