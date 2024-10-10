use std::{fs, process::Command};
use regex::Regex;
use sysinfo:: System;
use tokio::task::spawn_blocking;
use futures::future::join_all;

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
    pub async fn new() -> Self {
      let (username, hostname, distro, packages, kernel_version, init_system, uptime, de_wm, shell, cpu, gpu, memory, colors) = tokio::join!(
         Self::get_username(),
         Self::get_hostname(),
         Self::get_distro(),
         Self::get_package_info(),
         Self::get_kernel_version(),
         Self::get_init_system(),
         Self::get_uptime(),
         Self::get_de_wm(),
         Self::get_shell(),
         Self::get_cpu_info(),
         Self::get_gpu_info(),
         Self::get_memory_info(),
         Self::get_terminal_colors(),
      );

      SystemInfo {
         username,
         hostname,
         distro,
         packages,
         kernel_version,
         init_system,
         uptime,
         de_wm,
         shell,
         cpu,
         gpu,
         memory,
         colors,
      }
    }

    async fn get_username() -> String {
      spawn_blocking(|| std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())).await.unwrap()
    }

    async fn get_hostname() -> String {
      spawn_blocking(|| {
         Command::new("hostname")
         .output()
         .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
         .unwrap_or_else(|_| "unknown".to_string())
      }).await.unwrap()
    }

    async fn get_distro() -> String {
      spawn_blocking(|| {
         fs::read_to_string("/etc/os-release")
         .map(|content| {
            let re = Regex::new(r#"PRETTY_NAME="(.+)""#).unwrap();
            re.captures(&content)
               .and_then(|cap| cap.get(1))
               .map(|m| m.as_str().to_string())
               .unwrap_or_else(|| "unknown".to_string())
         })

         .unwrap_or_else(|_| "unknown".to_string())
      }).await.unwrap()
    }

    async fn get_package_info() -> PackageInfo {
         let package_managers = vec![
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

         let futures = package_managers.into_iter().map(|(cmd, pm)| {
            tokio::spawn(async move {
               let full_cmd = format!("command -v {} && {}", pm, cmd);

               let output = tokio::process::Command::new("sh")
                  .arg("-c")
                  .arg(&full_cmd)
                  .output()
                  .await;

               output.ok().and_then(|output| {
                  if output.status.success() {
                     String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .last()
                        .and_then(|line| line.trim().parse::<usize>().ok())
                  } else {
                     None
                  }
               })
            })
         });
   
         let results = join_all(futures).await;
         let native = results.into_iter().filter_map(|r| r.ok().flatten()).sum();
   
         let flatpak = tokio::process::Command::new("flatpak")
               .arg("list")
               .output()
               .await
               .map(|output| String::from_utf8_lossy(&output.stdout).lines().count())
               .unwrap_or(0);
   
         let snap = tokio::process::Command::new("snap")
               .arg("list")
               .output()
               .await
               .map(|output| String::from_utf8_lossy(&output.stdout).lines().count().saturating_sub(1))
               .unwrap_or(0);
   
            PackageInfo {
               native,
               flatpak,
               snap,
            }
    }

    async fn get_kernel_version() -> String {
      spawn_blocking(|| {
         Command::new("uname")
            .arg("-r")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string()) 
      }).await.unwrap()
    }

    async fn get_init_system() -> String {
      spawn_blocking(|| {
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
      }).await.unwrap()
    }

    async fn get_uptime() -> String {
      spawn_blocking(|| {
         let uptime = std::fs::read_to_string("/proc/uptime")
            .map(|content| content.split_whitespace().next().unwrap_or("0").parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

         let hours = ((uptime % 86400.0) / 3600.0) as u64;
         let minutes = ((uptime % 3600.0) / 60.0) as u64;

         format!("{}h ¦ {}m", hours, minutes)
      }).await.unwrap()
    }

    async fn get_de_wm() -> String {
      spawn_blocking(|| {
         std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| "unknown".to_string())
      }).await.unwrap()
    }

    async fn get_shell() -> String {
      spawn_blocking(|| {
         std::env::var("SHELL")
            .map(|s| s.split('/').last().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string())
      }).await.unwrap()
    }

    async fn get_cpu_info() -> String {
      spawn_blocking(|| {
         let mut sys = System::new_all();
         sys.refresh_all();
         let cpus = sys.cpus();

         if let Some(cpu) = cpus.first() {
            cpu.brand().to_string()
         } else {
            "unknown".to_string()
         }
      }).await.unwrap()
    }

    async fn get_gpu_info() -> String {
      spawn_blocking(|| {
         let output = Command::new("lspci")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .unwrap_or_else(|_| String::new());

         let re = Regex::new(r"(?i)VGA compatible controller: (.+)").unwrap();
         re.captures(&output)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
      }).await.unwrap()
    }

    async fn get_memory_info() -> MemoryInfo {
      spawn_blocking(|| {
         let mut sys = System::new_all();
         sys.refresh_all();

         MemoryInfo {
            used: sys.used_memory(),
            total: sys.total_memory(),
         }
      }).await.unwrap()
    }

    async fn get_terminal_colors() -> Vec<String> {
      spawn_blocking(|| {
         (30..38)
            .map(|i| format!("\x1b[{}m██\x1b[0m", i))
            .collect()
      }).await.unwrap()
    }
}