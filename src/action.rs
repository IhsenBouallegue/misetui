use crate::model::{
    ConfigFile, DetectedTool, DriftState, EditorState, EnvVar, InstalledTool, MiseProject,
    MiseSetting, MiseTask, OutdatedTool, PruneCandidate, RegistryEntry,
};

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
    OutdatedLoaded(Vec<OutdatedTool>),
    TasksLoaded(Vec<MiseTask>),
    EnvLoaded(Vec<EnvVar>),
    SettingsLoaded(Vec<MiseSetting>),
    PruneLoaded(Vec<PruneCandidate>),
    ToolInfoLoaded(String),
    ProjectsLoaded(Vec<MiseProject>),

    // Drift indicator
    CheckDrift,
    DriftChecked(DriftState),
    JumpToDriftProject,

    // Bootstrap Wizard
    WizardDetected(Vec<DetectedTool>),
    WizardToggleTool,
    WizardToggleAgentFiles,
    WizardNextStep,
    WizardPrevStep,
    WizardCompleted(String),

    // Inline Editor
    OpenEditor { path: String },
    EditorLoaded(Box<EditorState>),
    EditorSwitchTab,
    EditorStartEdit,
    EditorConfirmEdit,
    EditorCancelEdit,
    EditorDeleteRow,
    EditorAddTool,
    EditorAddEnvVar,
    EditorAddTask,
    EditorWrite,
    EditorWriteComplete(String),
    EditorClose,
    EditorInput(char),
    EditorBackspace,

    // Operations
    InstallTool,
    UninstallTool,
    UpdateTool,
    Confirm,
    CancelPopup,
    UpgradeAll,
    RunTask,
    UseTool,
    PruneTool,
    Refresh,
    TrustConfig,
    ShowToolDetail,
    InstallProjectTools { path: String },
    UpdateProjectPins { path: String },
    CycleSortOrder,
    OpenScanConfig,
    SaveScanConfig,

    // Mouse / popup search
    MouseClick { x: u16, y: u16 },
    PopupSearchInput(char),
    PopupSearchBackspace,

    // Status
    OperationComplete(String),
    OperationFailed(String),
    ShowHelp,

    // Internal
    Tick,
    Render,
    None,
}
