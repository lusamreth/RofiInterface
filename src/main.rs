use std::io::{stdout, Read, Result, Write};
use std::process::{self, Command, Stdio};
#[macro_use]
extern crate lazy_static;

mod lib;
mod nmcli;
mod var;
mod loopnm;

use std::str;
use tokio;

use lib::*;
use var::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //nmcli::run_rofi_nmcli().await?;
    //loopnm::loopnm(0);
    cont().await
}

fn search_rofi() {
    let mut a = Rofi::new("drun");
    a.locate(Position::Middle)
        .theme("/coding/system-testing/wifi.rasi")
        .justify_content(Direction::Center)
        .layout("475x5:10");
    a.finish();
}

async fn overall() {
    let mut a = Rofi::new("dmenu");
    a.locate(Position::Top);
    a.finish();
}

use std::env;
async fn cont() -> std::io::Result<()> {
    // args need to be

    let mut i = env::args().skip(1);

    let mut move_next = || i.next().unwrap_or("".to_string());

    let cmd = move_next();
    let option = move_next();

    match cmd.to_lowercase().trim() {
        "lockscreen" => {
            let mut default = true;
            if option.trim() == "cache" {
                default = false;
            }
            betterlockscreen(DEFAULT_WALLPAPER, default).await;
            Ok(())
        }
        "powermenu" => powerbutton().await,
        "nmcli" => nmcli::run_rofi_nmcli().await,
        "searcher" => {
            search_rofi();
            Ok(())
        }
        "help" => {
            help();
            Ok(())
        }
        /* Other rofi applications !*/
        _ => {
            println!("Invalid Command");
            help();
            Ok(())
        }
    }
}

fn help() {
    println!("Help: \nnmcli : Run Rofi with nmcli backend!\nsearcher: launcher with rofi!\nDev:lusamreth")
}

const POWERMENU: [&'static str; 4] = ["Hibernate", "Lock", "reboot", "Poweroff"];

// using better_lockscreen ! :
// betterlockscreen -s dim -u ~/Downloads/wallpaperbetter.jpg -b 1.5

async fn powerbutton() -> std::io::Result<()> {
    let txt = create_rofi_texts(POWERMENU.to_vec());
    let mut r = Rofi::new("dmenu");
    r.locate(Position::Top)
        .input(txt.stdout.unwrap())
        .margin_top(BARHEIGHT)
        .justify_content(Direction::Right)
        .layout("400x3");

    let c = r.finish().run().await?;
    let s = std::str::from_utf8(c.as_slice()).unwrap_or("");

    let pos = POWERMENU.iter().position(|x| x.trim() == s.trim());
    let loginctl = |arg| loginctl(arg,0);
    if pos.is_some() {
        match pos.unwrap() {
            0 => loginctl("suspend"),
            1 => betterlockscreen(DEFAULT_WALLPAPER, true).await,
            2 => loginctl("reboot"),
            3 => loginctl("poweroff"),
            _ => panic!("Out of reach!"),
        }
    }
    Ok(())
}

const BLUR: f32 = 0.5; // 0..1
const DEFAULT_WALLPAPER: &str = "/home/lusamreth/Downloads/wallpaperbetter.jpg";

use std::process::ExitStatus;
// require background path
async fn betterlockscreen(path: &str, cache: bool) {
    let mut cmd = tokio::process::Command::new("betterlockscreen");
    cmd.args(&["-s", "--blur"]);

    if !cache {
        cmd.arg("-u").arg(DEFAULT_WALLPAPER);
    }
    let mut c = cmd.args(&["-b", &BLUR.to_string()]).spawn().unwrap();

    if c.wait().await.unwrap().success() {
        println!("Lockscreen success!");
    }
}
use std::thread;
use std::time::Duration;
use std::sync::Mutex;

//static mut RESET: std::sync::Mutex<u32 > = Mutex::new(0);

fn loginctl(arg: &str,mut reset:u32) {
    let mut x = Command::new("loginctl");
    let mut c = x.arg(arg)
        .spawn()
        .expect("Cannot shutdown computer");
    let suc = c.wait().unwrap().success();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(MAXSLEEP));
        // if  failed to run within time , kill proc
        c.kill().expect("CANNOT kill process!");
    });

    if !suc{
        if reset == MAXRESET {
            println!("Unsuccessfull retries!");
            return;
        }
        println!("Re called {}", arg);
        // try shutting down xorg server
        reset += 1;
        loginctl(arg,reset);
    }
}

fn killorg(pid: &str) {
    Command::new("pkill")
        .arg("xorg")
        .spawn()
        .expect("Cannot Kill the process!");
}
