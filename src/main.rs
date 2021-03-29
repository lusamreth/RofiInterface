use std::io::{stdout, Read, Result, Write};
use std::process::{self, Command, Stdio};
#[macro_use]
extern crate lazy_static;

use std::str;
use tokio;
mod lib;
mod nmcli;
use lib::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //nmcli::run_rofi_nmcli().await?;
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

    match cmd.to_lowercase().trim(){
        "lockscreen" => {
            let mut default = true;
            if option.trim() == "cache" {
               default = false; 
            }
            betterlockscreen(DEFAULT_WALLPAPER,default).await;
            Ok(())
        },
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
use std::collections::HashMap;

const POWERMENU: [&'static str; 3] = ["Hibernate", "Lock", "Poweroff"];

// using better_lockscreen ! :
// betterlockscreen -s dim -u ~/Downloads/wallpaperbetter.jpg -b 1.5

async fn powerbutton() -> std::io::Result<()> {
    let txt = create_rofi_texts(POWERMENU.to_vec());
    let mut r = Rofi::new("dmenu");
    r.locate(Position::Top)
        .input(txt.stdout.unwrap())
        .justify_content(Direction::Right)
        .layout("400x3");

    let c = r.finish().run().await?;
    let s = std::str::from_utf8(c.as_slice()).unwrap_or("");

    let pos = POWERMENU.iter().position(|x| {
        x.trim() == s.trim()
    });

    if pos.is_some() {
        match pos.unwrap() {
            0 => suspend(),
            1 => betterlockscreen(DEFAULT_WALLPAPER,true).await,
            2 => {},
            _ => panic!("Out of reach!")
        }
    }
    Ok(())
}

const BLUR: f32 = 0.5; // 0..1
const DEFAULT_WALLPAPER : &str = "/home/lusamreth/Downloads/wallpaperbetter.jpg";

use std::process::ExitStatus;
// require background path
async fn betterlockscreen(path: &str,cache:bool) {

    let mut cmd = tokio::process::Command::new("betterlockscreen");
    cmd.args(&["-s", "--blur"]);

    if !cache {
       cmd.arg("-u")
        .arg(DEFAULT_WALLPAPER);
    }
    let mut c = cmd.args(&["-b", &BLUR.to_string()]).spawn().unwrap();
    
   if c.wait().await.unwrap().success() {
        println!("Lockscreen success!");
   }
}

fn suspend() {
    let mut c = Command::new("loginctl").arg("suspend").spawn().expect("Cannot suspend  computer");
    if !c.wait().unwrap().success(){
        println!("Re called suspension");
        suspend();
    }
}
