use crate::textfsm::errors::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum OptionType {
    Required,
    Filldown,
    Fillup,
    Key,
    List,
}

impl OptionType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Required" => Some(OptionType::Required),
            "Filldown" => Some(OptionType::Filldown),
            "Fillup" => Some(OptionType::Fillup),
            "Key" => Some(OptionType::Key),
            "List" => Some(OptionType::List),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueOption {
    pub option_type: OptionType,
    pub data: HashMap<String, String>,
}

impl ValueOption {
    pub fn new(option_type: OptionType) -> Self {
        Self {
            option_type,
            data: HashMap::new(),
        }
    }

    pub fn on_create_options(&mut self) -> Result<()> {
        match self.option_type {
            OptionType::Filldown => {
                self.data.insert("_myvar".to_string(), String::new());
            }
            OptionType::List => {
                self.data.insert("_value".to_string(), "[]".to_string());
            }
            _ => {}
        }
        Ok(())
    }

    pub fn on_assign_var(&mut self, value: &str) -> Result<()> {
        match self.option_type {
            OptionType::Filldown => {
                self.data.insert("_myvar".to_string(), value.to_string());
            }
            OptionType::Fillup => {
                // Store the value for fillup processing
                self.data
                    .insert("_fillup_value".to_string(), value.to_string());
            }
            OptionType::List => {
                // For List option, we append to the list
                let current = self.data.get("_value").unwrap_or(&"[]".to_string()).clone();
                let mut list: Vec<String> = serde_json::from_str(&current).unwrap_or_default();
                list.push(value.to_string());
                self.data
                    .insert("_value".to_string(), serde_json::to_string(&list)?);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn on_clear_var(&mut self, _current_value: &str) -> Result<String> {
        match self.option_type {
            OptionType::Filldown => Ok(self.data.get("_myvar").unwrap_or(&String::new()).clone()),
            OptionType::List => {
                self.data.insert("_value".to_string(), "[]".to_string());
                Ok(String::new())
            }
            _ => Ok(String::new()),
        }
    }

    pub fn on_clear_all_var(&mut self) -> Result<()> {
        match self.option_type {
            OptionType::Filldown => {
                self.data.insert("_myvar".to_string(), String::new());
            }
            OptionType::List => {
                self.data.insert("_value".to_string(), "[]".to_string());
            }
            _ => {}
        }
        Ok(())
    }

    pub fn on_save_record(&self, value: &str) -> Result<String> {
        match self.option_type {
            OptionType::Required => {
                if value.is_empty() {
                    return Err(crate::textfsm::errors::TextFSMError::FSMError(
                        "SkipRecord".to_string(),
                    ));
                }
                Ok(value.to_string())
            }
            OptionType::List => {
                let default_list = "[]".to_string();
                let list_str = self.data.get("_value").unwrap_or(&default_list);
                Ok(list_str.clone())
            }
            _ => Ok(value.to_string()),
        }
    }
}
