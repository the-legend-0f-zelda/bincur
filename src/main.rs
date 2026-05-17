use bincur::event::reactor::Reactor;


fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut reactor = Reactor::new(args);

    loop {
        match reactor.run() {
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                reactor.reset();
            }
            _ => return Ok(())
        }
    }
}
