use bincur::runtime::event::EventDriver;

fn main() -> std::io::Result<()> {
    let mut ed = EventDriver::new();
    ed.run()
}
