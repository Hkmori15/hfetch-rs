use std::{fs, process::Command};
use regex::Regex;
use sysinfo:: System;

pub struct SystemInfo {
   pub username: String,
   pub hostname: String,
   pub distro: String,
   pub packages: PackageInfo,
   pub kernel_version: String,
   pub init_system: String,
   pub uptime: String,
   pub de_wm: String,
   pub shell: String,
   pub cpu: String,
   pub gpu: String,
   pub memory: MemoryInfo,
   pub colors: Vec<String>,
}

pub struct PackageInfo {
   pub native: usize,
   pub flatpak: usize,
   pub snap: usize,
}

pub struct MemoryInfo {
   pub used: u64,
   pub total: u64,
}

impl SystemInfo {
    pub fn new() -> Self {
      let mut sys = System::new_all();
      sys.refresh_all();

      SystemInfo {
         username: Self::get_username(),
         hostname: Self::get_hostname(),
         distro: Self::get_distro(),
         packages: Self::get_package_info(),
         kernel_version: Self::get_kernel_version(),
         init_system: Self::get_init_system(),
         uptime: Self::get_uptime(),
         de_wm: Self::get_de_wm(),
         shell: Self::get_shell(),
         cpu: Self::get_cpu_info(&sys),
         gpu: Self::get_gpu_info(),
         memory: MemoryInfo {
            used: sys.used_memory(),
            total: sys.total_memory(),
         },

         colors: Self::get_terminal_colors(),
      }
    }

    fn get_username() -> String {
      std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
    }

    fn get_hostname() -> String {
      Command::new("hostname")
         .output()
         .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
         .unwrap_or_else(|_| "Unknown".to_string())
    }

    fn get_distro() -> String {
      fs::read_to_string("/etc/os-release")
         .map(|content| {
            let re = Regex::new(
               r#"PRETTY_NAME="(.+)""#).unwrap();
               re.captures(&content)
                  .and_then(|cap| cap.get(1))
                  .map(|m| m.as_str().to_string())
                  .unwrap_or_else(|| "Unknown".to_string())
         })

         .unwrap_or_else(|_| "Unknown".to_string())
    }

    fn get_package_info() -> PackageInfo {
      let package_managers = [
         ("dpkg-query -f '${binary:Package}\n' -W | wc -l", "dpkg"),
         ("dnf list installed | wc -l", "dnf"),
         ("zypper search --installed-only | wc -l", "zypper"),
         ("rpm -qa | wc -l", "rpm"),
         ("pacman -Q | wc -l", "pacman"),
         ("xbps-query -l | wc -l", "xbps-query"),
         ("brew list --formula | wc -l", "brew"),
         ("nix-env -q | wc -l", "nix-env"),
         ("ls -d /var/db/pkg/*/* | wc -l", "portage"),
         ("ls /var/log/packages/* | wc -l", "slackware"),
      ];

      let native = package_managers.iter()
         .find_map(|&(cmd, pm)| {
            let full_cmd = format!("command -v {} && {}", pm, cmd);

            Command::new("sh")
               .arg("-c")
               .arg(&full_cmd)
               .output()
               .ok()
               .and_then(|output| {
                  if output.status.success() {
                     String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .last()
                        .and_then(|line| line.trim().parse().ok())
                  } else {
                     None
                  }
               })
         })
         .unwrap_or(0);

         let flatpak = Command::new("flatpak")
            .arg("list")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).lines().count())
            .unwrap_or(0);

         let snap = Command::new("snap")
            .arg("list")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).lines().count().saturating_sub(1))
            .unwrap_or(0);

         PackageInfo {
            native,
            flatpak,
            snap,
         }
    }

    fn get_kernel_version() -> String {
      std::process::Command::new("uname")
         .arg("-r")
         .output()
         .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
         .unwrap_or_else(|_| "Unknown".to_string())
    }

    fn get_init_system() -> String {
      if std::path::Path::new("/run/systemd/system").exists() {
         "systemD".to_string()
      } else if std::path::Path::new("/sbin/openrc").exists() {
         "openRC".to_string()
      } else if std::path::Path::new("/usr/bin/runit").exists() {
         "runit".to_string()
      } else if std::path::Path::new("/bin/s6-svscan").exists() {
         "s6".to_string()
      } else if std::path::Path::new("/bin/dinit").exists() {
         "dinit".to_string()
      } else if std::path::Path::new("/sbin/init").exists() && std::fs::read_to_string("/sbin/init").map(|s| s.contains("sysvinit")).unwrap_or(false) {
         "sysvinit".to_string()
      } else if std::path::Path::new("/sbin/sinit").exists() {
         "sinit".to_string()
      } else if std::path::Path::new("/sbin/launchd").exists() {
         "launchd".to_string()
      } else {
         "Unknown".to_string()
      }
    }

    fn get_uptime() -> String {
      let uptime = std::fs::read_to_string("/proc/uptime")
         .map(|content| content.split_whitespace().next().unwrap_or("0").parse::<f64>().unwrap_or(0.0))
         .unwrap_or(0.0);

      let hours = ((uptime % 86400.0) / 3600.0) as u64;
      let minutes = ((uptime % 3600.0) / 60.0) as u64;

      format!("{}h ¦ {}m", hours, minutes)
    }

    fn get_de_wm() -> String {
      std::env::var("XDG_CURRENT_DESKTOP")
         .or_else(|_| std::env::var("DESKTOP_SESSION"))
         .unwrap_or_else(|_| "Unknown".to_string())
    }

    fn get_shell() -> String {
      std::env::var("SHELL")
         .map(|s| s.split('/').last().unwrap_or("Unknown").to_string())
         .unwrap_or_else(|_| "Unknown".to_string())
    }

    fn get_cpu_info(sys: &System) -> String {
      let cpus = sys.cpus();

      if let Some(cpu) = cpus.first() {
         cpu.brand().to_string()
      } else {
         "Unknown".to_string()
      }
    }

    fn get_gpu_info() -> String {
      let output = Command::new("lspci")
         .output()
         .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
         .unwrap_or_else(|_| String::new());

      let re = Regex::new(r"(?i)VGA compatible controller: (.+)").unwrap();
      re.captures(&output)
         .and_then(|cap| cap.get(1))
         .map(|m| m.as_str().trim().to_string())
         .unwrap_or_else(|| "Unknown".to_string())
    }

    fn get_terminal_colors() -> Vec<String> {
      (30..38)
         .map(|i| format!("\x1b[{}m██\x1b[0m", i))
         .collect()
    }
}