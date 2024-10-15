use core::fmt;
use std::{fs, process::Command};
use toml;
use serde::Deserialize;

const LINKER:&str="src/link_app.asm";

#[derive(Deserialize)]
struct Bin{
    name: String,
    start: String,
    end: String,
    file: String
}

#[derive(Deserialize)]
struct BinMap{
    bin: Vec<Bin>
}

fn main(){
    println!("cargo::rerun-if-changed=../user");
    let code = Command::new("make").current_dir("../user")
        .status().expect("ok");
    if !code.success(){
        panic!("makefile failed")
    }
    let s = std::fs::read_to_string("../user/binmap.toml").unwrap();
    let bin:BinMap = toml::from_str(&s).unwrap();
    let text = generate_linker(bin);
    fs::write(LINKER, text).unwrap();
}

fn generate_linker(binmap: BinMap)->String{
    let mut rt:String = String::new();
    write_linker(&mut rt, binmap).unwrap();
    rt
}

fn write_linker(rt: &mut String, binmap: BinMap)->fmt::Result{
    use std::fmt::Write;
    let apps = binmap.bin;
    writeln!(rt,r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
    apps.len()
    )?;

    for (idx,app) in apps.iter().enumerate(){
        let i = idx+1;
        writeln!(rt,r#"
    .quad app_{i}_name
    .quad app_{i}_start
    .quad app_{i}_end
    .quad {}
    .quad {}
    "#, app.start, app.end)?;
    }

    for (idx,app) in apps.iter().enumerate(){
        let i=idx+1;
        writeln!(rt, r#"
    .global app_{i}_name
    .global app_{i}_start
    .global app_{i}_end
app_{i}_name:
    .asciz "{}"
app_{i}_start:
    .incbin "../user/{}"
app_{i}_end:"#,
        app.name,
        app.file)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod test{
    use crate::{generate_linker, Bin, BinMap};

    #[test]
    fn test_generate_linker(){
        let rt = generate_linker(BinMap { bin: vec![
            Bin{
                name:"00name".to_string(),
                start:"0x01".to_string(),
                end:"0x02".to_string(),
                file: "target/file.bin".to_string()
            },
            Bin{
                name:"01name".to_string(),
                start:"0x01".to_string(),
                end:"0x02".to_string(),
                file: "target/file.bin".to_string()
            }
        ] });
        print!("{}",rt);
        assert_eq!(rt,r#"
        vv"#)
    }
}