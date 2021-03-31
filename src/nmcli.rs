use super::{create_rofi_texts, Direction, Position, Rofi};
use crate::var::*;
use async_recursion::async_recursion;
use lazy_static::lazy_static;
use std::convert::TryInto;
use std::process::{self as Stdprocess, Stdio};
use std::sync::Mutex;
use tokio::process;
const FIELDS: &'static str = "ssid,security";

lazy_static! {
    static ref BUFFER: Mutex<Vec<String>> = {
        let b = Vec::new();
        Mutex::new(b)
    };
}

#[async_recursion(?Send)]
pub async fn run_rofi_nmcli() -> std::io::Result<()> {
    let choice = retrieve_network().await;
    let ssid = fetch_ssid_only(choice.as_str());
    if ssid.is_empty() || ssid.to_lowercase() == "ssid" {
        Ok(())
    } else {
        let lock = BUFFER.lock().expect("Cannot hold buffer lock!");
        //ssid.to_string().retain(|s| s != ' ');
        let matched = lock.iter().find(|x| **x == ssid);
        if matched.is_none() {
            //drop mutex otherwise rofi won't render!
            //cause : retrieve_network() need to also hold mutex!
            drop(lock);
            let a = warning().await.unwrap().to_lowercase();
            if a.trim() == "return" {
                run_rofi_nmcli().await?
            }
        //retrieve_network();
        } else {
            println!("PRompting!");
            let p = prompt_password(&ssid).await;
            pass_display(p);
        }
        Ok(())
    }
}

async fn pass_display(sign: PassSig) {
    let suc = "Sucessfully authenticated!";
    let construct = |msg| {
        let mut base = Rofi::new("dmenu");
        base.message(msg)
            .margin_top(BARHEIGHT)
            .locate(NMPOS)
            .justify_content(NMDIR);
        base.finish();
    };
    match sign {
        PassSig::Next => construct(suc),
        PassSig::Cancel => (),
        PassSig::Failed => construct("Message Failed!"),
        PassSig::Return => {
            run_rofi_nmcli().await;
        }
    }
}

async fn warning() -> io::Result<String> {
    let ops = vec!["Return", "Exit"];

    let txt = create_rofi_texts(ops).stdout.unwrap();
    let mut rf = Rofi::new("dmenu:Ops");
    rf.margin_top(BARHEIGHT)
        .locate(NMPOS)
        .justify_content(NMDIR)
        .input(txt);
    //rf.theme("/coding/system-testing/wifi.rasi");
    rf.message("Invalid SSID! Please re-enter!");
    rf.layout("400x2:5:3");

    let output = rf.finish().run().await?;
    let res = std::str::from_utf8(output.as_slice()).unwrap();
    Ok(res.to_owned())
}

use std::io::{self, stdout, Read, Write};

const NETPROT: [&'static str; 2] = ["WPA", "WEP"];

fn fetch_ssid_only(line: &str) -> String {
    let mut res = String::new();
    let mut vc = line.trim().split_whitespace();

    loop {
        let back_str = vc.next_back();
        if back_str.is_none() {
            break;
        }
        let is_sec = NETPROT
            .iter()
            .find(|prot| back_str.unwrap().starts_with(*prot));
        if is_sec.is_none() {
            let txt = vc.clone().collect::<String>();
            res.push_str(txt.as_str());
            res.push_str(back_str.expect("Missing SSID!"));
            break;
        }
    }
    return res;
    //return is_sec.is_some();
}

use std::sync::mpsc;
use tokio::task;

async fn sample_rofi() {
    let mut f = Rofi::new("");
    f.sole_msg("working")
        .locate(NMPOS)
        .justify_content(NMDIR)
        .layout("400x1");
    let _x = f.finish();
}

use std::thread;
use std::time::Duration;

// to prevent blocking! use different thread
async fn retrieve_network() -> String {
    // require doas ;
    let mut list = process::Command::new("doas");
    list.arg("nice").arg("-n").arg("-20").arg("nmcli");
    list.arg("--field")
        .arg(FIELDS)
        .arg("device")
        .arg("wifi")
        .arg("list")
        .stdout(Stdio::piped());

    let ip = list.output().await.unwrap();
    let input_list = ip.stdout;
    let sr = std::str::from_utf8(input_list.as_slice());

    let sa = sr.unwrap().to_string();

    let iter_split = sa.trim_end().split("\n");

    let predict = iter_split
        .clone()
        .skip(1)
        .map(|line| fetch_ssid_only(line))
        .collect::<Vec<String>>();

    let v = iter_split.collect::<Vec<&str>>();
    let mut handle = BUFFER.lock().unwrap();

    if handle.len() > 0 {
        handle.clear();
    }

    predict.into_iter().for_each(|x| handle.push(x));
    let echo_txt = create_rofi_texts(v).stdout.unwrap();
    let i: Stdio = echo_txt.try_into().expect("Failed!");
    match process::Command::new("sed")
        .arg("/^--/d")
        .stdin(i)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
    {
        Some(output) => {
            let mut rofi_run = Rofi::new("dmenu:SSID");

            rofi_run
                .input(output)
                .locate(NMPOS)
                .margin_top(BARHEIGHT)
                .justify_content(NMDIR)
                .layout("400x5:12");

            match rofi_run.finish().run().await {
                Ok(res) => {
                    return std::str::from_utf8(res.as_slice()).unwrap().to_string();
                }
                Err(e) => {
                    eprintln!("running_error\n{:#?}", e);
                    Stdprocess::exit(1);
                }
            };
        }
        None => panic!("No output!"),
    }
}

enum PassSig {
    Cancel,
    Return,
    Next,
    Failed,
}

async fn prompt_password(ssid: &str) -> PassSig {
    // nmcli --ask device wifi connect $ssid
    let to_str = |op: Vec<u8>| {
        std::str::from_utf8(op.as_slice())
            .expect("Failed to convert!")
            .to_owned()
    };
    let mut rofi_run = Rofi::new("dmenu:Enter");

    let pass_option = create_rofi_texts(vec!["Return", "Cancel"]).stdout.unwrap();
    rofi_run
        .theme("/coding/system-testing/wifi.rasi")
        //.theme("solarized")
        .locate(NMPOS)
        .justify_content(NMDIR)
        .margin_top(BARHEIGHT)
        .input(pass_option)
        .layout("400x2:12");

    let input = rofi_run
        .finish()
        .run()
        .await
        .expect("Failed to fetch password!");

    let input = to_str(input);
    match input.as_str() {
        "Return" => PassSig::Return,
        "Cancel" => PassSig::Cancel,
        _ => {
            let nm_pipe = process::Command::new("nmcli")
                .args(&["device", "wifi"])
                .args(&["connect", ssid.trim()])
                .args(&["password", input.trim()])
                .stdout(Stdio::piped())
                .spawn()
                .expect("Bad pass!");

            let op = nm_pipe.wait_with_output().await.expect("No output");
            let opstr = to_str(op.stdout);

            println!("s {}", opstr);
            if opstr.find("success").is_some() {
                PassSig::Next
            } else {
                println!("Failed");
                PassSig::Failed
            }
        }
    }
}
