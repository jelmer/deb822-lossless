use debian_control::{Control, Priority};

pub fn main() {
    let mut control = Control::new();
    let mut source = control.add_source("hello");
    source.set_section("rust");

    let mut binary = control.add_binary("hello");
    binary.set_architecture(Some("amd64"));
    binary.set_priority(Priority::Optional);
    binary.set_description("Hello, world!");

    println!("{}", control);
}
