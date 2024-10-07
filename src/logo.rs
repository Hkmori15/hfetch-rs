use std::str::FromStr;

use colored::*;

use crate::config::Config;

pub struct Logo {
   content: Vec<String>,
   color: Color,
}

impl Logo {
    pub fn load(config: &Config) -> Self {
         let content = match &config.custom_logo {
             Some(logo) if !logo.is_empty() => logo.clone(),
             _ => Self::default_logo(),
         };

         let color = Color::from_str(&config.custom_logo_color).unwrap_or(Color::Magenta);

         Logo { content, color }
      }

      fn default_logo() -> Vec<String> {
           vec![
             ".------.",
             "|H.--. |",
             "| :/\\: |",
             "| (__) |",
             "| '--'H|",
             "`------'",
           ]
           .into_iter()
           .map(String::from)
           .collect()
        }

        pub fn display(&self) -> Vec<String> {
          self.content
             .iter()
             .map(|line| line.color(self.color).bold().to_string())
             .collect()
        }
   }


    
