use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum Language {
    #[default]
    System,
    PtBr,
    En,
    Es,
}

impl Language {
    pub const ALL: [Self; 4] = [Self::System, Self::PtBr, Self::En, Self::Es];

    pub fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::PtBr => "pt-BR",
            Self::En => "en",
            Self::Es => "es",
        }
    }

    /// A concrete language: an explicit choice wins, `System` follows `locale`.
    pub fn resolve(self, locale: &str) -> Self {
        if self != Self::System {
            return self;
        }
        let locale = locale.to_ascii_lowercase();
        if locale.starts_with("pt") {
            Self::PtBr
        } else if locale.starts_with("es") {
            Self::Es
        } else {
            Self::En
        }
    }

    /// Resolves against the desktop locale (LC_ALL, LC_MESSAGES, then LANG).
    pub fn current(self) -> Self {
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .into_iter()
            .filter_map(|name| std::env::var(name).ok())
            .find(|value| !value.is_empty())
            .unwrap_or_default();
        self.resolve(&locale)
    }
}

impl From<String> for Language {
    fn from(code: String) -> Self {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == code)
            .unwrap_or_default()
    }
}

impl From<Language> for String {
    fn from(language: Language) -> Self {
        language.code().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Subtitle,
    LanguageLabel,
    SystemLanguage,
    LanguageSaved,
    LocalFolder,
    Choose,
    ChooseLocalFolder,
    Accept,
    Cancel,
    RemoteLabel,
    RemoteFolderLabel,
    IntervalLabel,
    ExcludedFiles,
    ExcludedFolders,
    SaveSettings,
    OpenConfigFile,
    SyncNow,
    OpenApp,
    Quit,
    InitialStatus,
    TrayUnavailable,
    ConfigUnavailable,
    SettingsSaved,
    OpenEditorFailed,
    SavedConfigChanged,
    Syncing,
    WaitingToQuit,
    SyncDone,
    SyncFailed,
    UnsavedExternalChange,
    ReadConfigFailed,
    InvalidConfig,
    CreateConfigFolderFailed,
    EncodeConfigFailed,
    WriteConfigFailed,
    ReplaceConfigFailed,
    ConfigChangedExternally,
    InvalidLocalFolder,
    InvalidRemote,
    RemoteRootNotAllowed,
    IntervalTooShort,
    IntervalTooLong,
    AlreadyRunning,
    ProgramNotFound,
    StartFailed,
    SyncTab,
    AccountsTab,
    ConfiguredRemotes,
    Refresh,
    NoRemotes,
    NewDriveRemote,
    RemoteNameLabel,
    ClientIdLabel,
    ClientSecretLabel,
    CreateRemote,
    ClientIdHelp,
    Reconnect,
    Remove,
    ConfirmRemoveTitle,
    ConfirmRemoveBody,
    ContinueSetup,
    CancelSetup,
    Options,
    RunningRclone,
    WaitingBrowser,
    OperationCancelled,
    RemoteReady,
    RemoteRemoved,
    RcloneFailed,
    MalformedResponse,
    InvalidRemoteName,
    MissingCredentials,
    RemoteExists,
    RemoveIncomplete,
    OpenAuthLink,
}

/// Placeholders `{error}`, `{program}` and `{name}` are filled by callers with `str::replace`.
/// `System` is translated as English; resolve it first.
pub fn tr(language: Language, message: Message) -> &'static str {
    use Language::{En, Es, PtBr, System};
    use Message::*;
    let [pt, en, es] = match message {
        Subtitle => [
            "Espelhe uma pasta local para um destino privado do rclone.",
            "Mirror a local folder to a private rclone remote.",
            "Refleja una carpeta local en un destino privado de rclone.",
        ],
        LanguageLabel => ["Idioma", "Language", "Idioma"],
        SystemLanguage => ["Sistema", "System", "Sistema"],
        LanguageSaved => ["Idioma salvo.", "Language saved.", "Idioma guardado."],
        LocalFolder => ["Pasta local", "Local folder", "Carpeta local"],
        Choose => ["Escolher…", "Choose…", "Elegir…"],
        ChooseLocalFolder => [
            "Escolha a pasta local",
            "Choose the local folder",
            "Elige la carpeta local",
        ],
        Accept => ["Escolher", "Choose", "Elegir"],
        Cancel => ["Cancelar", "Cancel", "Cancelar"],
        RemoteLabel => ["Remoto rclone", "Rclone remote", "Remoto de rclone"],
        RemoteFolderLabel => ["Pasta no Drive", "Drive folder", "Carpeta en Drive"],
        IntervalLabel => ["Intervalo (min)", "Interval (min)", "Intervalo (min)"],
        ExcludedFiles => [
            "Arquivos ignorados (um padrão por linha)",
            "Ignored files (one pattern per line)",
            "Archivos ignorados (un patrón por línea)",
        ],
        ExcludedFolders => [
            "Pastas ignoradas (um padrão por linha)",
            "Ignored folders (one pattern per line)",
            "Carpetas ignoradas (un patrón por línea)",
        ],
        SaveSettings => [
            "Salvar configurações",
            "Save settings",
            "Guardar configuración",
        ],
        OpenConfigFile => [
            "Abrir arquivo de configuração",
            "Open config file",
            "Abrir archivo de configuración",
        ],
        SyncNow => ["Sincronizar agora", "Sync now", "Sincronizar ahora"],
        OpenApp => ["Abrir DriveMirror", "Open DriveMirror", "Abrir DriveMirror"],
        Quit => ["Sair", "Quit", "Salir"],
        InitialStatus => [
            "Configure o destino e salve antes da primeira sincronização.",
            "Set the destination and save before the first sync.",
            "Configura el destino y guarda antes de la primera sincronización.",
        ],
        TrayUnavailable => [
            "Bandeja indisponível: {error}. Mantenha esta janela aberta para controlar o app.",
            "Tray unavailable: {error}. Keep this window open to control the app.",
            "Bandeja no disponible: {error}. Mantén esta ventana abierta para controlar la app.",
        ],
        ConfigUnavailable => [
            "Configuração indisponível",
            "Configuration unavailable",
            "Configuración no disponible",
        ],
        SettingsSaved => [
            "Configurações salvas. O próximo ciclo usa o novo intervalo.",
            "Settings saved. The next cycle uses the new interval.",
            "Configuración guardada. El próximo ciclo usa el nuevo intervalo.",
        ],
        OpenEditorFailed => [
            "Não foi possível abrir o editor: {error}",
            "Could not open the editor: {error}",
            "No se pudo abrir el editor: {error}",
        ],
        SavedConfigChanged => [
            "A configuração salva mudou. Confira o destino exibido; o próximo ciclo usará estes valores.",
            "The saved configuration changed. Check the destination shown; the next cycle will use these values.",
            "La configuración guardada cambió. Revisa el destino mostrado; el próximo ciclo usará estos valores.",
        ],
        Syncing => ["Sincronizando…", "Syncing…", "Sincronizando…"],
        WaitingToQuit => [
            "Aguardando a sincronização terminar para sair com segurança…",
            "Waiting for the sync to finish to quit safely…",
            "Esperando a que termine la sincronización para salir con seguridad…",
        ],
        SyncDone => [
            "Sincronização concluída.",
            "Sync complete.",
            "Sincronización completada.",
        ],
        SyncFailed => ["Falha: {error}", "Failed: {error}", "Error: {error}"],
        UnsavedExternalChange => [
            "Há alterações não salvas e o arquivo mudou externamente; revise antes de sincronizar.",
            "There are unsaved changes and the file changed externally; review before syncing.",
            "Hay cambios sin guardar y el archivo cambió externamente; revisa antes de sincronizar.",
        ],
        ReadConfigFailed => [
            "Não foi possível ler a configuração: {error}",
            "Could not read config: {error}",
            "No se pudo leer la configuración: {error}",
        ],
        InvalidConfig => [
            "Configuração inválida: {error}",
            "Invalid config: {error}",
            "Configuración no válida: {error}",
        ],
        CreateConfigFolderFailed => [
            "Não foi possível criar a pasta de configuração: {error}",
            "Could not create config folder: {error}",
            "No se pudo crear la carpeta de configuración: {error}",
        ],
        EncodeConfigFailed => [
            "Não foi possível codificar a configuração: {error}",
            "Could not encode config: {error}",
            "No se pudo codificar la configuración: {error}",
        ],
        WriteConfigFailed => [
            "Não foi possível gravar a configuração: {error}",
            "Could not write config: {error}",
            "No se pudo escribir la configuración: {error}",
        ],
        ReplaceConfigFailed => [
            "Não foi possível substituir a configuração: {error}",
            "Could not replace config: {error}",
            "No se pudo reemplazar la configuración: {error}",
        ],
        ConfigChangedExternally => [
            "A configuração mudou externamente; revise-a antes de salvar",
            "The config changed externally; review it before saving",
            "La configuración cambió externamente; revísala antes de guardar",
        ],
        InvalidLocalFolder => [
            "Escolha uma pasta local absoluta existente",
            "Choose an existing absolute local folder",
            "Elige una carpeta local absoluta existente",
        ],
        InvalidRemote => [
            "Escolha um nome de remoto rclone válido",
            "Choose a valid rclone remote name",
            "Elige un nombre de remoto de rclone válido",
        ],
        RemoteRootNotAllowed => [
            "Escolha uma subpasta remota; a raiz do Drive não é permitida",
            "Choose a remote subfolder; the Drive root is not allowed",
            "Elige una subcarpeta remota; la raíz de Drive no está permitida",
        ],
        IntervalTooShort => [
            "O intervalo deve ser de pelo menos um minuto",
            "The interval must be at least one minute",
            "El intervalo debe ser de al menos un minuto",
        ],
        IntervalTooLong => [
            "O intervalo não pode passar de 24 horas",
            "The interval cannot exceed 24 hours",
            "El intervalo no puede superar las 24 horas",
        ],
        AlreadyRunning => [
            "Uma sincronização já está em andamento",
            "A synchronization is already running",
            "Ya hay una sincronización en curso",
        ],
        ProgramNotFound => [
            "{program} não foi encontrado; instale-o ou verifique o PATH",
            "{program} was not found; install it or check PATH",
            "No se encontró {program}; instálalo o revisa el PATH",
        ],
        StartFailed => [
            "Não foi possível iniciar {program}: {error}",
            "Could not start {program}: {error}",
            "No se pudo iniciar {program}: {error}",
        ],
        SyncTab => ["Sincronização", "Sync", "Sincronización"],
        AccountsTab => ["Contas rclone", "Rclone accounts", "Cuentas de rclone"],
        ConfiguredRemotes => [
            "Remotes configurados",
            "Configured remotes",
            "Remotos configurados",
        ],
        Refresh => ["Atualizar", "Refresh", "Actualizar"],
        NoRemotes => [
            "Nenhum remote configurado.",
            "No remotes configured.",
            "No hay remotos configurados.",
        ],
        NewDriveRemote => [
            "Novo remote Google Drive",
            "New Google Drive remote",
            "Nuevo remoto de Google Drive",
        ],
        RemoteNameLabel => ["Nome do remote", "Remote name", "Nombre del remoto"],
        ClientIdLabel => ["Client ID", "Client ID", "Client ID"],
        ClientSecretLabel => ["Client Secret", "Client Secret", "Client Secret"],
        CreateRemote => ["Criar remote", "Create remote", "Crear remoto"],
        ClientIdHelp => [
            "Como criar seu próprio Client ID do Google",
            "How to make your own Google Client ID",
            "Cómo crear tu propio Client ID de Google",
        ],
        Reconnect => ["Reconectar", "Reconnect", "Reconectar"],
        Remove => ["Remover", "Remove", "Eliminar"],
        ConfirmRemoveTitle => [
            "Remover o remote {name}?",
            "Remove the remote {name}?",
            "¿Eliminar el remoto {name}?",
        ],
        ConfirmRemoveBody => [
            "Isso remove apenas a configuração do rclone neste computador. Nenhum arquivo do Google Drive é apagado.",
            "This only removes the rclone configuration on this computer. No files on Google Drive are deleted.",
            "Esto solo elimina la configuración de rclone en este equipo. No se borra ningún archivo de Google Drive.",
        ],
        ContinueSetup => ["Continuar", "Continue", "Continuar"],
        CancelSetup => [
            "Cancelar configuração",
            "Cancel setup",
            "Cancelar configuración",
        ],
        Options => ["Opções", "Options", "Opciones"],
        RunningRclone => [
            "Executando o rclone…",
            "Running rclone…",
            "Ejecutando rclone…",
        ],
        WaitingBrowser => [
            "Aguardando autorização no navegador…",
            "Waiting for authorization in the browser…",
            "Esperando la autorización en el navegador…",
        ],
        OperationCancelled => [
            "Operação cancelada. Você pode tentar de novo.",
            "Operation cancelled. You can try again.",
            "Operación cancelada. Puedes intentarlo de nuevo.",
        ],
        RemoteReady => [
            "Remote {name} pronto para uso.",
            "Remote {name} is ready.",
            "El remoto {name} está listo.",
        ],
        RemoteRemoved => [
            "Remote {name} removido do rclone.",
            "Remote {name} removed from rclone.",
            "Remoto {name} eliminado de rclone.",
        ],
        RcloneFailed => [
            "O rclone falhou: {error}",
            "Rclone failed: {error}",
            "rclone falló: {error}",
        ],
        MalformedResponse => [
            "Resposta inválida do rclone; tente novamente.",
            "Invalid response from rclone; try again.",
            "Respuesta no válida de rclone; inténtalo de nuevo.",
        ],
        InvalidRemoteName => [
            "Nome de remote inválido: use letras, números, espaços internos e _ . + @ -",
            "Invalid remote name: use letters, digits, inner spaces and _ . + @ -",
            "Nombre de remoto no válido: usa letras, números, espacios internos y _ . + @ -",
        ],
        MissingCredentials => [
            "Informe o Client ID e o Client Secret.",
            "Enter the Client ID and Client Secret.",
            "Introduce el Client ID y el Client Secret.",
        ],
        RemoteExists => [
            "Já existe um remote chamado {name}.",
            "A remote named {name} already exists.",
            "Ya existe un remoto llamado {name}.",
        ],
        RemoveIncomplete => [
            "Remover remote incompleto {name}",
            "Remove incomplete remote {name}",
            "Eliminar remoto incompleto {name}",
        ],
        OpenAuthLink => [
            "Abrir link de autorização",
            "Open authorization link",
            "Abrir enlace de autorización",
        ],
    };
    match language {
        PtBr => pt,
        En | System => en,
        Es => es,
    }
}
