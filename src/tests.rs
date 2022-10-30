use super::logger;


#[test]
pub fn test() {
    use std::process::Command;
    use std::process::Stdio;
    use std::fs::File;
    
    /*let mut proc = Command::new("C:/nginx/nginx.exe")
    .current_dir("C:/nginx")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("puk"); */
    let file = File::create("out.txt").unwrap();
    let stdio = Stdio::from(file);
    
    let mut proc = Command::new("php.exe")
        .arg("C:/Users/defilak/Desktop/servicers/test/index.php")
        .arg("")
        .stdout(stdio)
        //.stderr(stdio)
        .spawn()
        .expect("puk");

    loop {
        match proc.try_wait() {
            Ok(Some(status)) => println!("exited with: {status}"),
            Ok(None) => {
                println!("status not ready yet, let's really wait");
                let res = proc.wait();
                println!("result: {res:?}");
            }
            Err(e) => println!("error attempting to wait: {e}"),
        }
    }
}

#[test]
fn asd() {
    logger::log("АХАХАХА БЛЯ АФОЛДФОЫАОДФЛЫ");
    logger::log("ПИЗДЕЦ");

    loop {} 
}