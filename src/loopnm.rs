use std::thread;
use std::time::Duration;
use std::process::Command;



pub fn loopnm(mut run : u32){
    if run > 5 {
        return;
    }
    // run every 20s
    thread::sleep(Duration::from_millis(20000));
    let c = Command::new("nmcli").arg("device").arg("wifi").arg("list").spawn().unwrap();
    
    let output = c.wait_with_output().unwrap().stdout;
    let strin = std::str::from_utf8(output.as_slice()).unwrap();

    println!("s {}",strin);

    run+=1;
    loopnm(run)
}
