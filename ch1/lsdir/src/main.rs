use std::fs::read_dir;
use std::io;

fn main() -> io::Result<()> {
	let dirents = read_dir(".")?;
    for entry in dirents{
        let entry = entry?;
        let epath = entry.path();
        let path = epath.to_str();
        match path{
            Some(s) => println!("{:?}", path),
            None => println!("bad path")
        }
    }
    let mut xs = vec![1,2,3,5];
    for x in xs.iter_mut(){
        println!("{}", x);
    }
    println!("{:?}", xs);
    Ok(())
}
