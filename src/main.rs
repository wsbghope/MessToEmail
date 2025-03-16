
use tao::{
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::macos::EventLoopExtMacOS,
};
rust_i18n::i18n!("locales");
use std::process::Command;
use tao::platform::macos::ActivationPolicy;
use tray_icon::{menu::MenuEvent, TrayIconEvent};

use MessToEmail::{
    get_sys_locale,check_full_disk_access,auto_launch,read_config,Config,auto_thread,
    TrayMenuItems,TrayMenu,TrayIcon,config_path
};

fn main() {
    let locale = get_sys_locale();
    rust_i18n::set_locale(locale);
    check_full_disk_access();
    let mut event_loop = EventLoopBuilder::new().build();
    event_loop.set_activation_policy(ActivationPolicy::Accessory);
    let auto = auto_launch();
    let mut config: Config = read_config();
    auto_thread();
    let tray_menu_items = TrayMenuItems::build(&config);
    let tray_menu = TrayMenu::build(&tray_menu_items);
    let mut tray_icon = TrayIcon::build(tray_menu);
    if config.hide_icon_forever {
        tray_icon.as_mut().unwrap().set_visible(false).expect("error");
    } else {
        tray_icon.as_mut().unwrap().set_visible(true).expect("error");
    }
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
        event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == tray_menu_items.quit_i.id() {
                tray_icon.take();
                *control_flow = ControlFlow::Exit;
            } else if event.id == tray_menu_items.check_hide_icon_for_now.id() {
                tray_icon.as_mut().unwrap().set_visible(false).expect("error");
            } else if event.id == tray_menu_items.check_hide_icon_forever.id() {
                config.hide_icon_forever = true;
                tray_icon.as_mut().unwrap().set_visible(false).expect("error");
                config.update().expect("eror");
            }  else if event.id == tray_menu_items.check_launch_at_login.id() {
                if tray_menu_items.check_launch_at_login.is_checked() {
                    let _ = auto.enable().is_ok();
                    if auto.is_enabled().unwrap() {
                        config.launch_at_login = true;
                        config.update().expect("eror");
                    } else {
                        tray_menu_items.check_launch_at_login.set_checked(false);
                    }
                } else {
                    let _ = auto.disable().is_ok();
                    if !auto.is_enabled().unwrap() {
                        config.launch_at_login = false;
                        config.update().expect("eror");
                    } else {
                        tray_menu_items.check_launch_at_login.set_checked(true);
                    }
                }
            } else if event.id == tray_menu_items.config.id() {
                println!("open config");
                Command::new("open")
                    .arg(config_path())
                    .output()
                    .expect("Failed to open config");
            } else {
                println!("what have you done?!");
            }
        }
        if let Ok(event) = tray_channel.try_recv() {
            println!("{event:?}");
        }
    });
}