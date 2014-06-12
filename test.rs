extern crate libc;
extern crate time;
#[allow(dead_code)]
mod nx;
fn load() -> nx::File {
    nx::File::open(&Path::new("Data.nx")).unwrap()
}
fn recurse(node: nx::Node) -> uint {
    node.iter().fold(1, |a, b| a + recurse(b))
}
fn str_recurse(node: nx::Node) -> uint {
    node.iter().fold(1, |a, b| {
        a + if node.get(b.name()).unwrap() == b { str_recurse(b) } else { 0 }
    })
}
fn test(name: &str, count: uint, func: || -> uint) {
    let mut answer = 0;
    let mut vec = Vec::from_fn(count, |_| {
        let begin = time::precise_time_ns();
        answer = func();
        let end = time::precise_time_ns();
        end - begin
    });
    vec.sort();
    let high = vec.get(vec.len() * 3 / 4);
    let slice = vec.slice(vec.len() * 1 / 4, vec.len() * 3 / 4);
    let mid = slice.iter().fold(0, |a, b| a + *b) / slice.len() as u64;
    let low = vec.get(0);
    println!("{}\t{}\t{}\t{}\t{}", name, high / 1000, mid / 1000, low / 1000, answer);
}
fn main() {
    unsafe { ::std::rt::stack::record_sp_limit(0); }
    let file = nx::File::open(&Path::new("Data.nx")).unwrap();
    let node = file.root();
    println!("Name\t75%t\tM50%\tBest\tAnswer");
    test("Ld", 0x1000, || load().header().nodecount as uint);
    test("Re", 0x20, || recurse(node));
    test("LR", 0x20, || recurse(load().root()));
    test("SA", 0x20, || str_recurse(node));
}
