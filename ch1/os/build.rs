use std::process::Command;

fn main(){
    println!("cargo::rerun-if-changed=../user");
    Command::new("make").current_dir("../user").args(&["all"])
        .status().unwrap();
}