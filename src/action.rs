use crate::model::{ConfigFile, InstalledTool, RegistryEntry};

#[derive(Debug, Clone)]
pub enum Action {
    // Navigation
    Quit,
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    NextTab,
    PrevTab,
    FocusSidebar,
    FocusContent,

    // Search
    EnterSearch,
    ExitSearch,
    SearchInput(char),
    SearchBackspace,

    // Data loaded
    ToolsLoaded(Vec<InstalledTool>),
    RegistryLoaded(Vec<RegistryEntry>),
    ConfigLoaded(Vec<ConfigFile>),
    DoctorLoaded(Vec<String>),
    VersionsLoaded(Vec<String>),

    // Operations
    InstallTool,
    UninstallTool,
    UpdateTool,
    ConfirmAction,
    CancelPopup,

    // Status
    OperationComplete(String),
    OperationFailed(String),
    ShowHelp,

    // Internal
    Tick,
    Render,
    None,
}
