use bincur::event::reactor::Reactor;


fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut reactor = Reactor::new(args);

    loop {
        if reactor.run().is_err() {reactor.reset();}
    }
}
