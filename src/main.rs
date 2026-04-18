use bincur::{device::keyboards::KEYBOARDS, runtime::event::EventDriver};

fn main() -> std::io::Result<()> {
    let mut ed = EventDriver::new();

    loop {
        let _r = ed.block_ready();
        for ev in ed.events.iter() {
            let token = ev.token();
            let kbd_idx = token.0;

            let mut guard = KEYBOARDS.lock().unwrap();
            let target = &mut guard.get_mut(kbd_idx).unwrap().1;

            loop {
                match target.fetch_events() {
                    Ok(iter) => for pressed in iter {
                        println!("입력: {:#?}", pressed);
                        //println!("타입: {}", pressed.)
                    },
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(e),
                }
            }
        }
    }
}
