#![feature(exact_size_is_empty)]
use std::fs;
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    if args.is_empty(){
        lsdir(".")
    }else{
        args.for_each(|dir|{
            lsdir(&dir)
        });
    }
}

fn lsdir(dir: &str){
    match fs::read_dir(dir){
        Ok(entries)=>{
            entries.for_each(|entry|{
                match entry {
                    Err(err)=>{
                        eprintln!("can not read entry {}, {:?}", dir, err)
                    },
                    Ok(entry)=>{
                        println!("{}", entry.path().to_str().unwrap());
                    }
                }
            });
        },
        Err(err)=>{
            eprintln!("can not read dir {}, {:?}", dir, err)
        }
    }
}
