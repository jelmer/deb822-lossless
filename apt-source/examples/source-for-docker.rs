use apt_source::Repositories;
use indoc::indoc;

pub const TEXT: &str = indoc! {r#"
    Types: deb
    URIs: https://download.docker.com/linux/ubuntu
    Suites: noble
    Components: stable
    Architectures: amd64
    Signed-By: /usr/share/keyrings/docker.gpg
"#};

pub fn main() {
    let c = TEXT.parse::<Repositories>().unwrap();
    //let license = c.find_license_for_file(Path::new("debian/foo")).unwrap();
    //println!("{}", license.name().unwrap());
}
