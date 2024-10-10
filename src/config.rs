// Not modifying this file
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use dirs;

#[derive(Serialize, Deserialize)]
pub struct Config {
   pub show_hostname: bool,
   pub show_distro: bool,
   pub show_packages: bool,
   pub show_kernel: bool,
   pub show_init_system: bool,
   pub show_uptime: bool,
   pub show_de_wm: bool,
   pub show_shell: bool,
   pub show_cpu: bool,
   pub show_gpu: bool,
   pub show_memory: bool,
   pub show_colors: bool,
   pub text_color: String,
   pub custom_logo: Option<Vec<String>>,
   pub custom_logo_color: String,
}

const DEFAULT_CONFIG: &str = 
r#" 
# ___  ___  ________ _______  _________  ________  ___  ___     
#|\  \|\  \|\  _____\\  ___ \|\___   ___\\   ____\|\  \|\  \    
#\ \  \\\  \ \  \__/\ \   __/\|___ \  \_\ \  \___|\ \  \\\  \   
# \ \   __  \ \   __\\ \  \_|/__  \ \  \ \ \  \    \ \   __  \  
#  \ \  \ \  \ \  \_| \ \  \_|\ \  \ \  \ \ \  \____\ \  \ \  \ 
#   \ \__\ \__\ \__\   \ \_______\  \ \__\ \ \_______\ \__\ \__\
#    \|__|\|__|\|__|    \|_______|   \|__|  \|_______|\|__|\|__|
#                          
                                             
# hfetch configuration file
show_hostname = true
show_distro = true
show_packages = true
show_kernel = true
show_init_system = true
show_uptime = true
show_de_wm = true
show_shell = true
show_cpu = false
show_gpu = false
show_memory = true
show_colors = true
text_color = "Magenta"

# Custom logo (leave empty to use default)
custom_logo = []
custom_logo_color = ""
"#;

impl Config {
    pub fn load() -> Self {
      let config_path = dirs::config_dir()
        .map(|mut path| {
          path.push("hfetch");
          path.push("config.toml");
          path
        })
        .unwrap_or_else(|| PathBuf::from("config.toml"));

      let config_str = match fs::read_to_string(&config_path) {
          Ok(content) => content,
          Err(_) => {
            // Create the default config file if config.toml don't exists
            if let Some(parent) = config_path.parent() {
              fs::create_dir_all(parent).expect("Failed to create config directory");
            }

            let mut file = fs::File::create(&config_path).expect("Failed to create config file");
            file.write_all(DEFAULT_CONFIG.as_bytes()).expect("Failed to write default config");
            DEFAULT_CONFIG.to_string()
          }
      };

      toml::from_str(&config_str).expect("Failed to parse config file")
    }
}