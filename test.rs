extern crate libc;
extern crate time;
mod nx;
fn recurse(node: nx::Node) -> int {
    node.iter().fold(1, |a, b| a + recurse(b))
}
fn test(name: &str, func: |nx::Node| -> int, node: nx::Node) {
    for _ in range(0, 10) {
        let begin = time::precise_time_ns();
        let answer = func(node);
        let end = time::precise_time_ns();
        println!("{}\t{}\t{}" , name , ( end - begin ) / 1000 , answer);
    }
}
fn main() {
    unsafe { ::std::rt::stack::record_sp_limit(0); }
    let file = nx::File::open(&Path::new("Data.nx")).unwrap();
    test("Re", recurse, file.root());
}
