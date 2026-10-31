use egui::{Context, Key, Modifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppShortcut {
    NewProject,
    OpenProject,
    SaveProject,
    SaveProjectAs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortcutBinding {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl AppShortcut {
    pub const ALL: [Self; 4] = [
        Self::NewProject,
        Self::OpenProject,
        Self::SaveProject,
        Self::SaveProjectAs,
    ];

    pub fn all() -> [Self; 4] {
        Self::ALL
    }

    /// Returns the exact keyboard binding for this action.
    pub fn binding(&self) -> ShortcutBinding {
        match self {
            Self::NewProject => ShortcutBinding {
                key: Key::N,
                modifiers: Modifiers::COMMAND,
            },
            Self::OpenProject => ShortcutBinding {
                key: Key::O,
                modifiers: Modifiers::COMMAND,
            },
            Self::SaveProject => ShortcutBinding {
                key: Key::S,
                modifiers: Modifiers::COMMAND,
            },
            Self::SaveProjectAs => ShortcutBinding {
                key: Key::S,
                modifiers: Modifiers::COMMAND.plus(Modifiers::SHIFT),
            },
        }
    }

    /// Platform-aware shortcut label (e.g. "Ctrl+N" on Windows/Linux, "Cmd+N" on macOS).
    pub fn label(&self) -> String {
        let is_mac = cfg!(target_os = "macos");
        let mod_str = if is_mac { "Cmd" } else { "Ctrl" };
        match self {
            Self::NewProject => format!("{}+N", mod_str),
            Self::OpenProject => format!("{}+O", mod_str),
            Self::SaveProject => format!("{}+S", mod_str),
            Self::SaveProjectAs => format!("{}+Shift+S", mod_str),
        }
    }

    /// Menu item display text with icon, action title, and synchronized shortcut label.
    pub fn menu_label(&self) -> String {
        match self {
            Self::NewProject => format!("📄  New Project ({})", self.label()),
            Self::OpenProject => format!("📂  Open .siro ({})", self.label()),
            Self::SaveProject => format!("💾  Save ({})", self.label()),
            Self::SaveProjectAs => format!("💾  Save As... ({})", self.label()),
        }
    }

    /// Platform-aware label for undo
    pub fn undo_label() -> String {
        let is_mac = cfg!(target_os = "macos");
        if is_mac { "Cmd+Z".to_string() } else { "Ctrl+Z".to_string() }
    }

    /// Platform-aware label for redo
    pub fn redo_label() -> String {
        let is_mac = cfg!(target_os = "macos");
        if is_mac { "Cmd+Y".to_string() } else { "Ctrl+Y".to_string() }
    }

    /// Platform-aware label for duplicate
    pub fn duplicate_label() -> String {
        let is_mac = cfg!(target_os = "macos");
        if is_mac { "Cmd+D".to_string() } else { "Ctrl+D".to_string() }
    }

    /// Returns true if this shortcut was pressed this frame (initial press, no repeat).
    pub fn is_triggered(&self, ctx: &Context) -> bool {
        let b = self.binding();
        ctx.input(|i| {
            i.events.iter().any(|event| {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } = event
                {
                    *key == b.key
                        && modifiers.command == b.modifiers.command
                        && modifiers.shift == b.modifiers.shift
                        && modifiers.alt == b.modifiers.alt
                } else {
                    false
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_bindings_and_labels() {
        let new_proj = AppShortcut::NewProject;
        assert_eq!(new_proj.binding().key, Key::N);
        assert!(new_proj.binding().modifiers.command);
        assert!(!new_proj.binding().modifiers.shift);

        let save_as = AppShortcut::SaveProjectAs;
        assert_eq!(save_as.binding().key, Key::S);
        assert!(save_as.binding().modifiers.command);
        assert!(save_as.binding().modifiers.shift);

        let is_mac = cfg!(target_os = "macos");
        let expected_prefix = if is_mac { "Cmd" } else { "Ctrl" };
        assert_eq!(new_proj.label(), format!("{}+N", expected_prefix));
        assert_eq!(save_as.label(), format!("{}+Shift+S", expected_prefix));
        assert!(new_proj.menu_label().contains(&new_proj.label()));
    }
}
