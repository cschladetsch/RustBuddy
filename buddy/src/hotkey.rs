use crate::config::HotkeyConfig;

#[cfg(target_os = "windows")]
use std::sync::Arc;

#[cfg(target_os = "windows")]
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Debug)]
pub enum HotkeyError {
    Parse(String),
    #[cfg(target_os = "windows")]
    Manager(global_hotkey::GlobalHotKeyError),
    #[cfg(target_os = "windows")]
    Register(global_hotkey::GlobalHotKeyError),
    Channel,
    Interrupt(std::io::Error),
}

impl std::fmt::Display for HotkeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(key) => write!(f, "invalid hotkey '{}'", key),
            #[cfg(target_os = "windows")]
            Self::Manager(err) => write!(f, "global hotkey manager error: {}", err),
            #[cfg(target_os = "windows")]
            Self::Register(err) => write!(f, "failed to register hotkey: {}", err),
            Self::Channel => write!(f, "hotkey event channel closed"),
            Self::Interrupt(err) => write!(f, "input interrupted: {}", err),
        }
    }
}

impl std::error::Error for HotkeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(target_os = "windows")]
            Self::Manager(err) | Self::Register(err) => Some(err),
            Self::Interrupt(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(target_os = "windows")]
pub struct HotkeyListener {
    rx: UnboundedReceiver<()>,
    _manager: Arc<global_hotkey::GlobalHotKeyManager>,
    _hotkey: global_hotkey::hotkey::HotKey,
}

#[cfg(not(target_os = "windows"))]
pub struct HotkeyListener {
    label: String,
}

#[cfg(target_os = "windows")]
pub fn parse_hotkey(
    hotkey: &str,
) -> Result<
    (
        global_hotkey::hotkey::Modifiers,
        global_hotkey::hotkey::Code,
    ),
    HotkeyError,
> {
    use global_hotkey::hotkey::{Code, Modifiers};

    let mut modifiers = Modifiers::empty();
    let mut code = None;
    for token in hotkey.split('+') {
        let token = token.trim().to_lowercase();
        match token.as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            other => {
                code =
                    Some(parse_code(other).ok_or_else(|| HotkeyError::Parse(other.to_string()))?);
            }
        }
    }
    let code = code.ok_or_else(|| HotkeyError::Parse("missing key".into()))?;
    Ok((modifiers, code))
}

#[cfg(target_os = "windows")]
fn parse_code(key: &str) -> Option<global_hotkey::hotkey::Code> {
    use global_hotkey::hotkey::Code;
    match key {
        "a" => Some(Code::KeyA),
        "b" => Some(Code::KeyB),
        "c" => Some(Code::KeyC),
        "d" => Some(Code::KeyD),
        "e" => Some(Code::KeyE),
        "f" => Some(Code::KeyF),
        "g" => Some(Code::KeyG),
        "h" => Some(Code::KeyH),
        "i" => Some(Code::KeyI),
        "j" => Some(Code::KeyJ),
        "k" => Some(Code::KeyK),
        "l" => Some(Code::KeyL),
        "m" => Some(Code::KeyM),
        "n" => Some(Code::KeyN),
        "o" => Some(Code::KeyO),
        "p" => Some(Code::KeyP),
        "q" => Some(Code::KeyQ),
        "r" => Some(Code::KeyR),
        "s" => Some(Code::KeyS),
        "t" => Some(Code::KeyT),
        "u" => Some(Code::KeyU),
        "v" => Some(Code::KeyV),
        "w" => Some(Code::KeyW),
        "x" => Some(Code::KeyX),
        "y" => Some(Code::KeyY),
        "z" => Some(Code::KeyZ),
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
impl HotkeyListener {
    pub fn new(cfg: &HotkeyConfig) -> Result<Self, HotkeyError> {
        use global_hotkey::hotkey::HotKey;
        use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

        let (modifiers, code) = parse_hotkey(&cfg.key)?;
        let hotkey = HotKey::new(Some(modifiers), code);
        let manager = Arc::new(GlobalHotKeyManager::new().map_err(HotkeyError::Manager)?);
        manager.register(hotkey).map_err(HotkeyError::Register)?;
        let (tx, rx) = mpsc::unbounded_channel();
        let global_event = GlobalHotKeyEvent::new();
        let hotkey_id = hotkey.id();
        std::thread::spawn(move || {
            while let Ok(evt) = global_event.receiver().recv() {
                if evt.id == hotkey_id {
                    let _ = tx.send(());
                }
            }
        });
        Ok(Self {
            rx,
            _manager: manager,
            _hotkey: hotkey,
        })
    }

    pub async fn wait(&mut self) -> Result<(), HotkeyError> {
        self.rx.recv().await.ok_or(HotkeyError::Channel)
    }
}

#[cfg(not(target_os = "windows"))]
impl HotkeyListener {
    pub fn new(cfg: &HotkeyConfig) -> Result<Self, HotkeyError> {
        Ok(Self {
            label: cfg.key.clone(),
        })
    }

    pub async fn wait(&mut self) -> Result<(), HotkeyError> {
        println!("Press Enter to simulate hotkey '{}'", self.label);
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(HotkeyError::Interrupt)?;
        Ok(())
    }
}
