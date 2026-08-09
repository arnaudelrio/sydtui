use std::collections::HashMap;

use crossterm::event::KeyCode;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Represents the key bindings for the application.
#[derive(Debug, Clone)]
pub struct Keybindings {
    /// Key codes for exiting the application.
    pub exit: Vec<KeyCode>,
    /// Key codes for displaying the help message.
    pub help: Vec<KeyCode>,
    /// Key codes for submitting a form.
    pub enter: Vec<KeyCode>,
    /// Key codes for saving the application state.
    pub save: Vec<KeyCode>,
    /// Key codes for reloading the application state.
    pub reload: Vec<KeyCode>,
    /// Key codes for moving to the previous selected item.
    pub previous: Vec<KeyCode>,
    /// Key codes for moving to the next selected item.
    pub next: Vec<KeyCode>,
    /// Key codes for creating a new item.
    pub new: Vec<KeyCode>,
    /// Key codes for renaming an item.
    pub rename: Vec<KeyCode>,
    /// Key codes for duplicating an item.
    pub duplicate: Vec<KeyCode>,
    /// Key codes for deleting an item.
    pub delete: Vec<KeyCode>,
    /// Key codes for toggling the activation state of an item.
    pub toggle_activate: Vec<KeyCode>,
    /// Key codes for toggling the enabled state of an item.
    pub toggle_enabled: Vec<KeyCode>,
    /// Key codes for viewing the logs.
    pub view_logs: Vec<KeyCode>,
}

impl Keybindings {
    /// Prints the help message for the key bindings.
    pub fn print_help(&self) -> String {
        let mut help = String::new();
        macro_rules! add_keybinding_help {
            ($help:expr, $($field:ident),*) => {
                $(
                    let to_str = |k: &KeyCode| match k {
                        KeyCode::Char(c) => c.to_string(),
                        KeyCode::Esc => "esc".to_string(),
                        KeyCode::Enter => "enter".to_string(),
                        KeyCode::Up => "up".to_string(),
                        KeyCode::Down => "down".to_string(),
                        KeyCode::Left => "left".to_string(),
                        KeyCode::Right => "right".to_string(),
                        KeyCode::Backspace => "backspace".to_string(),
                        KeyCode::Tab => "tab".to_string(),
                        KeyCode::F(n) => format!("f{n}"),
                        _ => format!("{k:?}").to_lowercase(),
                    };
                    let keys: Vec<String> = self.$field.iter().map(to_str).collect();
                    $help.push_str(&format!(" {}: [{}]\n", stringify!($field), keys.join(", ")));
                )*
            };
        }

        add_keybinding_help!(
            help,
            exit,
            help,
            enter,
            save,
            reload,
            previous,
            next,
            new,
            rename,
            duplicate,
            delete,
            toggle_activate,
            toggle_enabled,
            view_logs
        );

        help
    }
}

impl Default for Keybindings {
    /// Returns the default key bindings for the application.
    fn default() -> Self {
        Self {
            exit: vec![KeyCode::Char('q'), KeyCode::Esc],
            help: vec![KeyCode::Char('?')],
            enter: vec![KeyCode::Enter],
            save: vec![KeyCode::Char('s')],
            reload: vec![KeyCode::Char('f')],
            previous: vec![KeyCode::Up],
            next: vec![KeyCode::Down],
            new: vec![KeyCode::Char('n')],
            rename: vec![KeyCode::Char('r')],
            duplicate: vec![KeyCode::Char('y')],
            delete: vec![KeyCode::Char('d')],
            toggle_activate: vec![KeyCode::Char(' ')],
            toggle_enabled: vec![KeyCode::Char('e')],
            view_logs: vec![KeyCode::Char('l')],
        }
    }
}

impl Serialize for Keybindings {
    /// Serializes the key bindings to a JSON string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let to_str = |k: &KeyCode| match k {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Up => "up".to_string(),
            KeyCode::Down => "down".to_string(),
            KeyCode::Left => "left".to_string(),
            KeyCode::Right => "right".to_string(),
            KeyCode::Backspace => "backspace".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::F(n) => format!("f{n}"),
            _ => format!("{k:?}").to_lowercase(),
        };

        let mut map: HashMap<&str, Vec<String>> = HashMap::new();
        map.insert("exit", self.exit.iter().map(to_str).collect());
        map.insert("help", self.help.iter().map(to_str).collect());
        map.insert("enter", self.enter.iter().map(to_str).collect());
        map.insert("save", self.save.iter().map(to_str).collect());
        map.insert("reload", self.reload.iter().map(to_str).collect());
        map.insert("previous", self.previous.iter().map(to_str).collect());
        map.insert("next", self.next.iter().map(to_str).collect());
        map.insert("new", self.new.iter().map(to_str).collect());
        map.insert("rename", self.rename.iter().map(to_str).collect());
        map.insert("duplicate", self.duplicate.iter().map(to_str).collect());
        map.insert("delete", self.delete.iter().map(to_str).collect());
        map.insert(
            "toggle_activate",
            self.toggle_activate.iter().map(to_str).collect(),
        );
        map.insert(
            "toggle_enabled",
            self.toggle_enabled.iter().map(to_str).collect(),
        );
        map.insert("view_logs", self.view_logs.iter().map(to_str).collect());

        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Keybindings {
    /// Deserializes the key bindings from a JSON string.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, Vec<String>>::deserialize(deserializer)?;

        let parse_keys = |key_name: &str| -> Result<Vec<KeyCode>, D::Error> {
            let vec = map.get(key_name).ok_or_else(|| {
                de::Error::missing_field(Box::leak(key_name.to_owned().into_boxed_str()))
            })?;

            vec.iter()
                .map(|s| match s.to_lowercase().as_str() {
                    "esc" | "escape" => Ok(KeyCode::Esc),
                    "enter" | "return" => Ok(KeyCode::Enter),
                    "up" => Ok(KeyCode::Up),
                    "down" => Ok(KeyCode::Down),
                    "left" => Ok(KeyCode::Left),
                    "right" => Ok(KeyCode::Right),
                    "backspace" => Ok(KeyCode::Backspace),
                    "tab" => Ok(KeyCode::Tab),
                    s if s.len() == 1 => Ok(KeyCode::Char(s.chars().next().unwrap())),
                    s if s.starts_with('f') && s[1..].parse::<u8>().is_ok() => {
                        Ok(KeyCode::F(s[1..].parse::<u8>().unwrap()))
                    }
                    _ => Err(de::Error::custom(format!("unknown key binding: {s}"))),
                })
                .collect()
        };

        Ok(Keybindings {
            exit: parse_keys("exit")?,
            help: parse_keys("help")?,
            enter: parse_keys("enter")?,
            save: parse_keys("save")?,
            reload: parse_keys("reload")?,
            previous: parse_keys("previous")?,
            next: parse_keys("next")?,
            new: parse_keys("new")?,
            rename: parse_keys("rename")?,
            duplicate: parse_keys("duplicate")?,
            delete: parse_keys("delete")?,
            toggle_activate: parse_keys("toggle_activate")?,
            toggle_enabled: parse_keys("toggle_enabled")?,
            view_logs: parse_keys("view_logs")?,
        })
    }
}
