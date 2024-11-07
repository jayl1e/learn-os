#![no_std]
#![no_main]

use core::str;

use user_lib::*;

const READBUF_LIMIT:usize=128;
#[no_mangle]
fn main() -> i32 {
    let mut readbuf: [u8;READBUF_LIMIT]= [0u8;READBUF_LIMIT];
    loop {
        print!("shell $ ");
        let cmd = readline(&mut readbuf[..READBUF_LIMIT-1]);
        if cmd.is_none(){
            continue;
        }
        let cmd = cmd.unwrap();
        if cmd == "exit"{
            break;
        }else{
            let code = exec_cmd(cmd);
            println!("[shell] program exit with code: {}", code);
        }
    }
    0
}

fn exec_cmd(cmd: &str)->i32{
    match fork(){
        0=>{
            let code = exec(cmd);
            println!("exec cmd {} err code {}", cmd, code);
            exit(code as i32);
            panic!("should exit")
        },
        pid=>{
            let mut rcode = 0;
            wait4(pid as usize, &mut rcode);
            rcode
        }
    }


}

fn readline(buf: &mut[u8])->Option<&str>{
    let mut wptr = 0;
    for _ in 0..buf.len(){
        let c = get_char().unwrap();
        match c{
            b'\n' | b'\r' => {
                put_char(b'\n');
                break;
            },
            b'\x04'=>{
                buf[..4].copy_from_slice("exit".as_bytes());
                wptr = 4;
                break;
            },
            b'\x03'=>{
                return None;
            },
            b'\x7F'=>{
                if wptr>0{
                    wptr -= 1;
                    put_char(c);
                }
            }
            _=>{
                buf[wptr]=c;
                put_char(c);
                wptr+=1;
            }
        }
    }
    if wptr>=buf.len(){
        println!("[shell] input too long");
        return None
    }
    match  str::from_utf8(&buf[..wptr]){
        Err(e)=>{
            println!("[shell] readline error at {}", e.valid_up_to());
            None
        },
        Ok(v)=>{
            Some(v)
        }
        
    }
}
