use dionysus_core;

fn main() {
    let desktop_files = dionysus_core::get_dot_desktop_files();
    for file in desktop_files.iter() {
        print!("{:?}, ", file.file_name().unwrap());
    }
}
