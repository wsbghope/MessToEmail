
use std::{
    error::Error,
    fs,
    path::{PathBuf},
    process::Command,
    thread,
    time::Duration,
    fs::OpenOptions,
    io::Write
};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message,
    SmtpTransport,
    Transport,
};
use chrono:: Local;
use home::home_dir;
use native_dialog::{MessageDialog, MessageType};
use rust_i18n::t;
use auto_launch::AutoLaunch;
rust_i18n::i18n!("locales");
use sys_locale;
use serde::{Deserialize, Serialize};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    TrayIconBuilder,
};


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub sender : String,
    pub recipient :String,
    pub emailauthcode :String,
    pub hide_icon_forever: bool,
    pub launch_at_login: bool,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            sender: "".to_string(),
            recipient: "".to_string(),
            emailauthcode: "".to_string(),
            hide_icon_forever: false,
            launch_at_login: false,
        }
    }
}

impl Config {

    pub fn update(&self) -> Result<(), Box<dyn Error>> {
        let updated_config_str = serde_json::to_string(&self)?;
        fs::write(config_path(), updated_config_str)?;
        Ok(())
    }
}

pub fn config_path() -> PathBuf {
    let mut config_path = home_dir().unwrap();
    config_path.push(".config");
    config_path.push("messtoemail");
    config_path.push("messtoemail.json");
    config_path

}

pub fn log_path() -> PathBuf {
    let mut log_path = home_dir().unwrap();
    log_path.push(".config");
    log_path.push("messtoemail");
    log_path.push("messtoemail.log");
    if !log_path.exists() {
        fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    }
    log_path
}
pub fn write_log(info: &str) {
    let fmt = "%Y年%m月%d日 %H:%M:%S";
    let now = Local::now().format(fmt).to_string().replace("\n","");
        if !log_path().exists() {
        let str = String::new();
        fs::create_dir_all(log_path().parent().unwrap()).unwrap();
        fs::write(log_path(), str).unwrap();
    }
    let mut file = OpenOptions::new().append(true).open(log_path()).expect("cannot open file");
    let errinfo = format!("{}::{}",now,info);
    file.write_all(errinfo.as_bytes()).expect("write failed");
}

pub fn read_config() -> Config {
    if !config_path().exists() {
        let config = Config::default();
        let config_str = serde_json::to_string(&config).unwrap();
        fs::create_dir_all(config_path().parent().unwrap()).unwrap();
        fs::write(config_path(), config_str).unwrap();
    }

    let config_str = fs::read_to_string(config_path()).unwrap();
    let config = serde_json::from_str(&config_str);
    if config.is_err() {
        let config = Config::default();
        let config_str = serde_json::to_string(&config).unwrap();
        std::fs::write(config_path(), config_str).unwrap();
        return config;
    } else {
        return config.unwrap();
    }
}


pub struct TrayMenuItems {
    pub quit_i: MenuItem,
    pub check_hide_icon_for_now: MenuItem,
    pub check_hide_icon_forever: MenuItem,
    pub check_launch_at_login: CheckMenuItem,
    pub config: MenuItem,
}

impl TrayMenuItems {
    pub fn build(config: &Config) -> Self {
        let quit_i = MenuItem::new(t!("quit"), true, None);
        let check_hide_icon_for_now = MenuItem::new(t!("hide-icon-for-now"), true, None);
        let check_hide_icon_forever = MenuItem::new(t!("hide-icon-forever"), true, None);
        let check_launch_at_login =
            CheckMenuItem::new(t!("launch-at-login"), true, config.launch_at_login, None);
        let config = MenuItem::new(t!("config"), true, None);
        TrayMenuItems {
            quit_i,
            check_hide_icon_for_now,
            check_hide_icon_forever,
            check_launch_at_login,
            config,
        }
    }
}

pub struct TrayMenu {}

impl TrayMenu {
    pub fn build(tray_menu_items: &TrayMenuItems) -> Menu {
        let tray_menu = Menu::new();
        let _ = tray_menu.append_items(&[
            &Submenu::with_items(
                t!("hide-icon"),
                true,
                &[
                    &tray_menu_items.check_hide_icon_for_now,
                    &tray_menu_items.check_hide_icon_forever,
                ],
            )
            .expect("create submenu failed"),
            &tray_menu_items.check_launch_at_login,
            &PredefinedMenuItem::separator(),
            &tray_menu_items.config,
            &PredefinedMenuItem::separator(),
            &tray_menu_items.quit_i,
        ]);
        tray_menu
    }
}

pub struct TrayIcon {}

impl TrayIcon {
    pub fn build(tray_menu: Menu) -> Option<tray_icon::TrayIcon> {
        Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_title("📨")
                .build()
                .unwrap(),
        )
    }
}


pub fn get_current_exe_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    if path.to_str().unwrap().contains(".app") {
        path = path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
    }
    path
}

pub fn auto_launch() -> AutoLaunch {
    let app_name = env!("CARGO_PKG_NAME");
    let app_path = get_current_exe_path();
    println!("app_name: {:?}", app_name);
    println!("app_path: {:?}", app_path);
    let args = &["--minimized", "--hidden"];
    AutoLaunch::new(app_name, app_path.to_str().unwrap(), false, args)
}

pub fn check_full_disk_access() {
    let check_db_path = home_dir()
        .expect("获取用户目录失败")
        .join("Library/Messages");
    let ct = fs::read_dir(check_db_path);
    match ct {
        Err(_) => {
            let yes = MessageDialog::new()
                .set_type(MessageType::Info)
                .set_title(t!("full-disk-access").as_str())
                .show_confirm()
                .unwrap();
            if yes {
                Command::new("open")
                    .arg("/System/Library/PreferencePanes/Security.prefPane/")
                    .output()
                    .expect("Failed to open Disk Access Preferences window");
            }
            panic!("exit without full disk access");
        }
        _ => {}
    }
}

pub fn get_sys_locale() -> &'static str {
    let syslocal = sys_locale::get_locale().unwrap();
    let lang_code = &syslocal[0..2];
    match lang_code {
        "zh" => "zh-CN",
        "en" => "en",
        _ => "en",
    }
}

pub fn send_email(content: &str) -> Result<Message, Box<dyn std::error::Error>> {
        let config = read_config();
    let email_sender = config.sender;
    let email_recipient = config.recipient;
    let smtp_server = "smtp.qq.com";
    let password = config.emailauthcode;
    let email = Message::builder()
        .from(email_sender.parse().map_err(|e| format!("发件人解析失败: {}", e))?)
        .to(email_recipient.parse().map_err(|e| format!("收件人解析失败: {}", e))?)
        .subject("验证码已到")
        .header(ContentType::TEXT_PLAIN)
        .body(content.to_string())
        .map_err(|e| format!("邮件体构建失败: {}", e))?;
       let creds = Credentials::new(email_sender, password);
    let mailer = SmtpTransport::relay(smtp_server)
        .map_err(|e| format!("解析SMTP服务器失败: {}", e))
        .expect("解析SMTP服务器失败:")
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(email),
        Err(e) => panic!("发送邮件失败: {e:?}"),
    }

}

pub fn get_message_in_one_minute() -> String {

    let output = Command::new("sqlite3")
        .arg(home_dir().expect("获取用户目录失败").join("Library/Messages/chat.db"))
        .arg("SELECT text FROM message WHERE datetime(date/1000000000 + 978307200,\"unixepoch\",\"localtime\") > datetime(\"now\",\"localtime\",\"-60 second\") ORDER BY date DESC LIMIT 1;")
        .output()
        .expect("sqlite命令运行失败");
    let stdout = String::from_utf8(output.stdout).unwrap();
    return stdout;
}
pub fn auto_thread() {
    thread::spawn(move || {
        let check_db_path = home_dir().unwrap().join("Library/Messages/chat.db-wal");
        let mut last_metadata_modified = fs::metadata(&check_db_path).unwrap().modified().unwrap();
        let mut old_stdout = String::new();
        loop {
            let now_metadata = fs::metadata(&check_db_path).unwrap().modified().unwrap();
            if now_metadata != last_metadata_modified {
                last_metadata_modified = now_metadata;
                let stdout = get_message_in_one_minute();
                let stdout = stdout.trim().to_string();
                if stdout != old_stdout && !stdout.is_empty(){
                    match send_email(stdout.as_str()) {
                        Ok(..)=>{
                            let okinfo = format!("邮件发送成功:{}。\n",stdout);
                            write_log(okinfo.as_str());
                        },
                        Err(e)=>{
                            let errinfo = format!("邮件发送失败:{}。\n",e);
                            write_log(errinfo.as_ref());
                            eprintln!("邮件发送失败: {}\n", e);
                            println!("{}", errinfo)
                        }
                    }
                }
                old_stdout =stdout;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}
