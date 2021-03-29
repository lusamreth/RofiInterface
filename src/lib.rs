use std::convert::TryInto;
use std::io::{stdout, Read, Write};
use std::process::Stdio;
use std::str;
use tokio::process::{self, Command};
use tokio::task;

pub struct Rofi {
    args: Vec<String>,
    location: u8,
    mode: String,
    message: String,
    pipeline: Option<Stdio>,
}

pub struct RofiChild {
    stdout: std::io::Stdout,
    pub handler: process::Child,
}

pub enum Position {
    Top,
    Middle,
    Bottom,
}

pub enum Direction {
    Left,
    Right,
    Center,
}
struct Location {
    position: Position,
    direction: Direction,
}

// define location
//  1 2 3
//  8 0 4
//  7 6 5

impl Rofi {
    pub fn new(mode: &str) -> Self {
        let mut args = Vec::new();
        let mut m;

        if mode.is_empty() == false {
            let mut s = mode.split(":");
            let mut mode = s.next().unwrap_or("").to_string();

            let prompt = s.next().unwrap_or("");
            if mode == "dmenu" {
                mode.insert(0, '-');
            }
            m = mode;
            args.push("-p".to_string());
            args.push(prompt.to_string());
        } else {
            // default is message mode
            m = "Message".to_string();
        }
        Rofi {
            args,
            location: 99, // empty
            mode: m,
            message: String::new(),
            pipeline: None,
        }
    }

    // This message is printed with the search menu!
    pub fn message(&mut self, msg: &str) -> &mut Self {
        self.args.push("-mesg".to_string());
        self.args.push(msg.to_string());
        self
    }

    // This is just solely a message and no entry!
    pub fn sole_msg(&mut self, m: &str) -> &mut Self {
        self.message.push_str(m);
        self
    }
    pub fn locate(&mut self, p: Position) -> &mut Self {
        let l_num = match p {
            Position::Top => 1,
            Position::Middle => 8,
            Position::Bottom => 7,
        };
        self.location = l_num;
        self
    }

    pub fn add_config_file(&mut self, path: &str) -> &mut Self {
        self.args.push("-config".to_string());
        self.args.push(path.to_string());
        self
    }

    pub fn layout(&mut self, l_input: &str) -> &mut Self {
        // standard display input : widthxlines+padding
        if l_input.to_lowercase() == "fullscreen" || l_input == "f" {
            self.args.push("-fullscreen".to_string());
            return self;
        }
        let mut s = l_input.splitn(2, |c| c == 'x' || c == '+');
        let badformat = "Error bad format!";
        let i1 = s.next().expect(badformat);
        let i2 = s.next().expect(badformat);
        let inputs = vec!["-width", i1, "-lines", i2];

        for i in inputs.into_iter() {
            self.args.push(i.to_string());
        }

        self
    }

    pub fn justify_content(&mut self, dir: Direction) -> &mut Self {
        let lnum = self.location;
        if lnum == 99 {
            panic!("Please justify the position first!")
        }
        // default location is center = 0;
        let jsf = |dnum: [u8; 3]| match lnum {
            1 => dnum[0],
            8 => dnum[1],
            7 => dnum[2],
            _ => panic!("Not a default location!"),
        };
        self.location = match dir {
            Direction::Center => jsf([2, 0, 6]),
            Direction::Right => jsf([3, 4, 5]),
            Direction::Left => jsf([1, 8, 7]),
        };

        self
    }

    fn hori(&mut self, num: i32) {
        self.args.push("-yoffset".to_string());
        self.args.push(num.to_string());
    }

    fn vert(&mut self, num: i32) {
        self.args.push("-xoffset".to_string());
        self.args.push(num.to_string());
    }

    pub fn margin_top(&mut self, num: u32) -> &mut Self {
        self.hori(num as i32);
        self
    }

    pub fn margin_bottom(&mut self, num: u32) -> &mut Self {
        let num = (num) as i32 * -1;
        self.hori(num);
        self
    }

    pub fn margin_right(&mut self, num: u32) -> &mut Self {
        let num = (num) as i32 * -1;
        self.vert(num);
        self
    }

    pub fn margin_left(&mut self, num: u32) -> &mut Self {
        self.vert(num as i32);
        self
    }

    pub fn input(&mut self, inp: process::ChildStdout) -> &mut Self {
        let i = inp.try_into().expect("Error converting to stdio");
        self.pipeline = Some(i);
        self
    }

    pub fn theme(&mut self, path: &str) -> &mut Self {
        self.args.push("-theme".to_string());
        self.args.push(path.to_string());
        self
    }

    // build the rofi child!
    pub fn finish(self) -> RofiChild {
        let mut cmd = Command::new("rofi");
        cmd.kill_on_drop(true);
        if self.mode.as_str() != "Message" {
            cmd.arg("-show");
            cmd.arg(self.mode);
        } else {
            cmd.arg("-e");
            cmd.arg(self.message);
        }

        cmd.arg("-location").arg(self.location.to_string());
        cmd.args(self.args);
        cmd.stdout(Stdio::piped());

        match self.pipeline {
            Some(input) => {
                cmd.stdin(input);
            }
            None => (),
        };

        let child = cmd.spawn().expect("Cannot spawn rofi child!");
        let stdout = stdout();
        RofiChild {
            stdout,
            handler: child,
        }
    }
}

impl RofiChild {
    // consume the lock!
    pub async fn run(self) -> std::io::Result<Vec<u8>> {
        let renderer = self.stdout.lock();
        let output = self.handler.wait_with_output().await?.stdout;
        return Ok(output);
    }

    pub async fn id(&self) -> Option<u32> {
        self.handler.id()
    }

    pub async fn wait(&mut self) {
        self.handler.wait().await;
    }
}

pub fn create_rofi_texts(cmds: Vec<&str>) -> process::Child {
    let options = cmds.iter().map(|cmd| {
        let mut s = cmd.split(":");
        let name = s.next().unwrap_or("");
        let cmd = s.next().unwrap_or("");
        return (name, cmd);
    });
    let names = options
        .enumerate()
        .map(|(i, (s1, _))| {
            let mut s1 = s1.to_string();
            if i != cmds.len() - 1 {
                s1.push('\n');
            }
            s1
        })
        .collect::<String>();

    let filter_options = process::Command::new("echo")
        .arg("-e")
        .arg(names)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Cannot fetch options");

    return filter_options;
}

//#[tokio::test(flavor = "multi_thread", worker_threads = 1)]A
#[tokio::test]
async fn testing() {
    let mut r = Rofi::new("dmenu");
    let echo_child = Command::new("echo")
        .arg("-e")
        .arg("apple\nsauce\ncan\npancake")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let echo_input = echo_child.stdout.expect("Cannot get output");
    r.margin_top(10)
        .layout("400x4+10")
        .locate(Position::Top)
        .justify_content(Direction::Right);

    r.input(echo_input);
    let rofi_child = r.finish();
    //let (tx,rx) = mpsc::channel();
    let i = rofi_child.run().await.unwrap();
    let o = str::from_utf8(i.as_slice()).expect("Cannot convert!");
    let o = o.trim();
    assert!(o == "apple" || o == "sauce" || o == "can" || o == "pancake");
}
