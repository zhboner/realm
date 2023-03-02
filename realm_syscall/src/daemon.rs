use std::env::current_dir;
use daemonize::Daemonize;

/// Daemonize.
///
/// Fork the process in the background,
/// disassociate from process group and the control terminal.
///
/// Keep current working directory,
/// redirect all standard streams to `/dev/null`.
///
/// Finally, print a message if succeeds or an error occurs.
#[cfg(unix)]
pub fn daemonize(msg: &'static str) {
    let pwd = current_dir().unwrap().canonicalize().unwrap();

    let daemon = Daemonize::new().umask(0).working_directory(pwd);

    match daemon.start() {
        Ok(_) => println!("{}", msg),
        Err(e) => eprintln!("failed to daemonize: {}", e),
    }
}
