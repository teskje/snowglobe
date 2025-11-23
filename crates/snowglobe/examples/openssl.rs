fn main() {
    let mut buf = [0; 10];
    openssl::rand::rand_bytes(&mut buf).unwrap();
    print!("{buf:?}");
}
