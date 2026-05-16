use bincur::event::reactor::Reactor;


fn main() -> std::io::Result<()> {
    let mut reactor = Reactor::new();
    let args: Vec<String> = std::env::args().skip(1).collect();

    loop {
        match reactor.run(&args) {
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                reactor.reset();
            }
            _ => return Ok(())
        }
    }
}
