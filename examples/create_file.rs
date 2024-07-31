// create a file

fn main() {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create("foo.txt").expect("Unable to create file");
    file.write_all(b"Hello, world!")
        .expect("Unable to write data");
}
