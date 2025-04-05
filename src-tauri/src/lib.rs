use inject_lib::{Data, Inject, Injector};
use serde::Serialize;
use tempfile::NamedTempFile;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs, path::Path, sync::Mutex};
use tauri::State;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

#[derive(Debug)]
struct Gta5Injector<'a> {
    yim_dll_path: Mutex<String>,
    fsl_dll_path: Mutex<String>,
    process_name: &'a str,
    gta5: Option<Gta5>,
}

#[derive(Debug, Serialize, Clone)]
struct Gta5 {
    platform: GamingPlatform,
    game_path: String,
    fsl_path: Option<PathBuf>,
    is_running: bool,
}

#[derive(Debug, Serialize, Clone)]
enum GamingPlatform {
    Steam,
    Rockstar,
    Epic,
}

impl Gta5 {
    pub fn build() -> Result<Gta5, String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let rockstar_reg = hklm
            .open_subkey("SOFTWARE\\WOW6432Node\\Rockstar Games")
            .map_err(|e| format!("未找到 Rockstar 注册项: {:?}", e))?;
        let mut gta5_reg = None;
        for ele in rockstar_reg.enum_keys() {
            let ele = ele.map_err(|e| format!("获取 Rockstar 注册表项时出错: {:?}", e))?;
            if ele.contains("GTA") && ele.contains("V") && ele.contains("Enhanced") {
                gta5_reg = Some(
                    rockstar_reg
                        .open_subkey(ele)
                        .map_err(|e| format!("未找到 GTA5 注册项: {:?}", e))?,
                );
                break;
            }
        }
        let gta5_reg = gta5_reg.ok_or("未找到 GTA5 注册表项".to_string())?;
        for result in gta5_reg.enum_values() {
            let (name, value) = result.map_err(|e| format!("获取 GTA5 注册表项时出错: {:?}", e))?;
            if name.contains("InstallFolder") {
                let platform = if name.contains("Steam") {
                    GamingPlatform::Steam
                } else if name.contains("Epic") {
                    GamingPlatform::Epic
                } else {
                    GamingPlatform::Rockstar
                };

                let game_path = value.to_string();

                if !Path::new(&format!("{}\\GTA5_Enhanced.exe", game_path)).exists() {
                    return Err("未找到 GTA5".to_string());
                };

                let fsl_path = PathBuf::from(&format!("{}\\version.dll", game_path));
                let fsl_path = if fsl_path.exists() {
                    Some(fsl_path)
                } else {
                    None
                };

                let is_running = match Injector::find_pid(Data::Str("GTA5_Enhanced.exe")) {
                    Ok(pids) => !pids.is_empty(),
                    Err(_) => false,
                };

                return Ok(Gta5 {
                    platform,
                    game_path,
                    fsl_path,
                    is_running,
                });
            }
        }

        Err("未找到 GTA5 注册表项".to_string())
    }
}

impl<'a> Gta5Injector<'a> {
    fn new() -> Self {
        Self {
            yim_dll_path: Mutex::new("assets\\YimMenuV2.dll".to_string()),
            fsl_dll_path: Mutex::new("assets\\version.dll".to_string()),
            process_name: "GTA5_Enhanced.exe",
            gta5: Gta5::build().inspect_err(|e| println!("{}", e)).ok(),
        }
    }

    fn inject_gta5(&self) -> Result<(), String> {
        // 查找进程 PID
        let pid = match Injector::find_pid(Data::Str("GTA5_Enhanced.exe")) {
            Ok(pids) => {
                if pids.len() == 0 {
                    return Err(format!("未找到 {} 进程", self.process_name).to_string());
                }
                pids[0]
            }
            Err(e) => {
                return Err(format!("查找进程 PID 时出错: {:?}", e));
            }
        };

        // 获取 DLL 路径
        let dll_path = &*self
            .yim_dll_path
            .lock()
            .map_err(|err| format!("获取 DLL 路径时出错 {:?}", err))?;

        // 创建 Injector 对象
        let dll_path = Data::Path(Path::new(dll_path));
        let injector = Injector::new(dll_path, pid);

        // 注入 DLL
        let result = injector.inject(true).inject();
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("注入 DLL 时出错: {:?}", e)),
        }
    }

    fn get_dll_path(&self) -> Result<String, String> {
        let dll_path = &*self
            .yim_dll_path
            .lock()
            .map_err(|err| format!("获取 DLL 路径时出错 {:?}", err))?;

        fs::canonicalize(Path::new(dll_path))
            .map_err(|e| format!("获取 DLL 路径时出错: {}", e))
            .and_then(|path| {
                path.to_str()
                    .map(|s| s.to_string())
                    .ok_or("获取 DLL 路径时出错".to_string())
            })
    }

    fn install_fsl(&self) -> Result<(), String> {
        let fsl_path = &*self
            .fsl_dll_path
            .lock()
            .map_err(|err| format!("获取 FSL DLL 路径时出错 {:?}", err))?;

        let fsl_path = Path::new(fsl_path);
        if !fsl_path.exists() {
            return Err("FSL DLL 文件不存在".to_string());
        }

        let gta5_path = &self
            .gta5
            .as_ref()
            .ok_or("GTA5 信息未初始化".to_string())?
            .game_path;

        let fsl_dst_path = Path::new(gta5_path).join("version.dll");

        let mut temp_file = NamedTempFile::new_in(&gta5_path).map_err(|e| format!("创建临时文件时出错: {}", e) )?;

        let fsl_file = File::open(fsl_path).map_err(|e| format!("打开 FSL DLL 文件时出错: {}", e))?;
        let mut reader = BufReader::new(fsl_file);

        let mut writer = BufWriter::new(temp_file.as_file_mut());

        std::io::copy(&mut reader, &mut writer).map_err(|e| format!("复制 FSL DLL 文件时出错: {}", e))?;

        writer.flush().map_err(|e| format!("刷新缓冲区时出错: {}", e))?;

        drop(reader);
        drop(writer);
        temp_file.persist(fsl_dst_path.to_str().ok_or("GTA5 信息未初始化".to_string())?).map_err(|e| format!("保存临时文件时出错: {}", e))?;

        Ok(())
    }

    fn run_gta5(&self) -> Result<(), String> {
        let plat = &self
            .gta5
            .as_ref()
            .ok_or("GTA5 信息未初始化".to_string())?
            .platform;

        let gta5_path = &self
            .gta5
            .as_ref()
            .ok_or("GTA5 信息未初始化".to_string())?
            .game_path;

            match plat {
                GamingPlatform::Steam => {
                    open::that("steam://rungameid/3240220")
                        .map_err(|e| format!("启动 GTA5 时出错: {}", e))?;
                }
                GamingPlatform::Epic => {
                    open::with("com.epicgames.launcher://apps/b0cd075465c44f87be3b505ac04a2e46%3A122e5e90b7b8424d930be8bc1a7e05fb%3A8769e24080ea413b8ebca3f1b8c50951?action=launch&silent=true", "explorer")
                        .map_err(|e| format!("启动 GTA5 时出错: {}", e))?;
                }
                _ => {
                    Command::new(gta5_path)
                        .spawn()
                        .map_err(|e| format!("启动 GTA5 时出错: {}", e))?;
                }
                
            }

        Ok(())
    }

    fn update_gta5_info(&mut self) -> Result<(), String> {
        self.gta5 = match Gta5::build() {
            Ok(gta5) => Some(gta5),
            Err(e) => return Err(e),
        };
        Ok(())
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn inject_gta5(state: State<'_, Mutex<Gta5Injector>>) -> Result<(), String> {
    let injector = state.lock().map_err(|e| format!("锁定 Gta5Injector 时出错: {:?}", e))?;
    Ok(injector.inject_gta5()?)
}

#[tauri::command]
fn get_gta5_info(state: State<'_, Mutex<Gta5Injector>>) -> Result<Gta5, String> {
    let mut injector = state.lock().map_err(|e| format!("锁定 Gta5Injector 时出错: {:?}", e))?;
    injector.update_gta5_info()?;
    injector.gta5.clone().ok_or("获取 GTA5 信息时出错".to_string())
}

#[tauri::command]
fn get_dll_path(state: State<'_, Mutex<Gta5Injector>>) -> Result<String, String> {
    let injector = state.lock().map_err(|e| format!("锁定 Gta5Injector 时出错: {:?}", e))?;
    Ok(injector.get_dll_path()?)
}

#[tauri::command]
fn install_fsl(state: State<'_, Mutex<Gta5Injector>>) -> Result<(), String> {
    let injector = state.lock().map_err(|e| format!("锁定 Gta5Injector 时出错: {:?}", e))?;
    Ok(injector.install_fsl()?)
}

#[tauri::command]
fn run_gta5(state: State<'_, Mutex<Gta5Injector>>) -> Result<(), String> {
    let injector = state.lock().map_err(|e| format!("锁定 Gta5Injector 时出错: {:?}", e))?;
    Ok(injector.run_gta5()?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(Gta5Injector::new()))
        .invoke_handler(tauri::generate_handler![inject_gta5, get_dll_path, get_gta5_info, install_fsl, run_gta5])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用程序时出错");
}

#[cfg(test)]
mod tests {
    use crate::{GamingPlatform, Gta5};

    #[test]
    fn test_gta5_build() {
        let gta5 = Gta5::build().unwrap();
        assert!(matches!(gta5.platform, GamingPlatform::Epic));
        assert_eq!(gta5.game_path, "D:\\Program Files\\Epic Games\\GTAVEnhanced");
        assert_eq!(gta5.fsl_path.unwrap().to_str().unwrap(), "D:\\Program Files\\Epic Games\\GTAVEnhanced\\version.dll");
    }
}
