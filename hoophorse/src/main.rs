use engine;

fn main() {
    let fb2d = engine::Fb2d::new((255, 255, 255, 255));
    engine::main(fb2d);
    println!("Hello, world!");
}
