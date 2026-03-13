// NOTE: I actually don't need this for anything :crying:
// app.connect_activate() and app.connect_startup() do the job
// I am keeping it nonetheless, maybe I will need some IPC for something else ???

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

pub const TOGGLE_COMMAND: &str = "toggle";

// A helper to send out a toggle dispatch
pub fn send_toggle() -> io::Result<()> {
    let path = socket_path();
    let mut stream = UnixStream::connect(&path)?;
    stream.write_all(TOGGLE_COMMAND.as_bytes())?;
    stream.write_all(b"\n")?;
    Ok(())
}

// The function that will initiate the listening on the dionysus.sock
// Takes in a closure that will be called on every new line received by the socket
pub fn start_listener<F>(mut handler: F) -> io::Result<()>
where
    F: FnMut(String) + Send + 'static,
{
    let path = socket_path();
    let listener = bind_listener(&path)?;

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(_) => continue,
            };

            let mut buffer = String::new();
            if stream.read_to_string(&mut buffer).is_err() {
                continue;
            }

            for line in buffer.lines() {
                handler(line.trim().to_string());
            }
        }
    });

    Ok(())
}

// "Binding" both creates the socket and returns a listener
// If it was already there and we can connect, no need to do anything
// But if no one is listening, just get rid of the stale one and bind a new one
fn bind_listener(path: &Path) -> io::Result<UnixListener> {
    if path.exists() {
        if UnixStream::connect(path).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                "socket already in use",
            ));
        }
        let _ = std::fs::remove_file(path);
    }

    UnixListener::bind(path)
}

// Puts dionysus socket in the DXG default runtime dir where most sockets live
// thats /run/user/1000 most of the time
fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR");
    let runtime_dir = runtime_dir.unwrap_or_else(|| OsString::from("/tmp"));
    PathBuf::from(runtime_dir).join("dionysus.sock")
}
