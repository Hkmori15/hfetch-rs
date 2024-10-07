mod config;
mod system_info;
mod logo;

use std::str::FromStr;

use colored::*;

use config::Config;
use regex::Regex;
use system_info::SystemInfo;
use logo::Logo;

fn main() {
   let config = Config::load();
   let system_info = SystemInfo::new();
   let logo = Logo::load(&config);

   print_info(&config, &system_info, &logo);
}

fn strip_ansi_codes(s: &str) -> String {
  let re = Regex::new(r"\x1B\[[0-9;]*[mGK]").unwrap();
  re.replace_all(s, "").to_string()
}

fn print_info(config: &Config, system_info: &SystemInfo, logo: &Logo) {
    let color = Color::from_str(&config.text_color).unwrap_or(Color::White);

    let logo_lines = logo.display();
    let mut info_lines = Vec::new();

    if config.show_hostname {
      info_lines.push(format!("{}: {}¦{}", "hostname".color(color).bold(), system_info.username, system_info.hostname));
    }

    if config.show_distro {
      info_lines.push(format!("{}: {}", "distro".color(color).bold(), system_info.distro));
    }

    if config.show_packages {
      info_lines.push(format!("{}: {} (native: {} ¦ flatpak: {} ¦ snap: {})", "packages".color(color).bold(), system_info.packages.native + system_info.packages.flatpak + system_info.packages.snap, system_info.packages.native, system_info.packages.flatpak, system_info.packages.snap));
    }

    if config.show_kernel {
      info_lines.push(format!("{}: {}", "kernel".color(color).bold(), system_info.kernel_version));
    }

    if config.show_init_system {
      info_lines.push(format!("{}: {}", "init".color(color).bold(), system_info.init_system));
    }

    if config.show_uptime {
      info_lines.push(format!("{}: {}", "uptime".color(color).bold(), system_info.uptime));
    }

    if config.show_de_wm {
      info_lines.push(format!("{}: {}", "de|wm".color(color).bold(), system_info.de_wm));
    }

    if config.show_shell {
      info_lines.push(format!("{}: {}", "shell".color(color).bold(), system_info.shell));
    }

    if config.show_cpu {
      info_lines.push(format!("{}: {}", "cpu".color(color).bold(), system_info.cpu));
    }

    if config.show_gpu {
      info_lines.push(format!("{}: {}", "gpu".color(color).bold(), system_info.gpu));
    }

    if config.show_memory {
      let used_gb = system_info.memory.used as f64 / 1_073_741_824.0;
      let total_gb = system_info.memory.total as f64 / 1_073_741_824.0;
      info_lines.push(format!("{}: {:.1} GB ¦ {:.1} GB", "memory".color(color).bold(), used_gb, total_gb));
    }

    if config.show_colors {
      let colors_label = "colors".color(color).bold();
      let color_blocks = system_info.colors.join("");

      info_lines.push(format!("{}: {}", colors_label, color_blocks));
    }

    let max_logo_width = logo_lines.iter().map(|line| strip_ansi_codes(line).len()).max().unwrap_or(0);
    let _max_info_length = info_lines.iter().map(|line| line.len()).max().unwrap_or(0);

    for (i, logo_line) in logo_lines.iter().enumerate() {
      if i < info_lines.len() {
         let info_line = &info_lines[i];
         println!("{:width$}  {}", logo_line, info_line, width = max_logo_width + logo_line.len() - strip_ansi_codes(logo_line).len());
      } else {
         println!("{}", logo_line);
      }
    }

    for line in info_lines.iter().skip(logo_lines.len()) {
      println!("{:width$}  {}", "", line, width = max_logo_width);
    }
   
}