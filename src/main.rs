use git2::Repository;

fn main() {
    let repo = match Repository::open("/home/kami/Documents/Coding/Rust/gitwatch") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    println!("{:?}", repo.state());
}
