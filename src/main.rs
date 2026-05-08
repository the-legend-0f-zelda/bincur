use bincur::runtime::event::EventDriver;

fn main() -> std::io::Result<()> {
    let mut ed = EventDriver::new();
    loop {
        match ed.run() {
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                ed.reset();
            }
            _ => return Ok(())
        }
    }
}
