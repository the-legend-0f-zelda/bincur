use bincur::{device::keyboards::KEYBOARDS, runtime::event::EventDriver, setup::keymap};

fn main() -> std::io::Result<()> {
    let mut ed = EventDriver::new();
    keymap::load();

    loop {
        let _r = ed.block_ready();
        for ev in ed.events.iter() {
            let token = ev.token();
            let kbd_idx = token.0;

            KEYBOARDS.with_borrow_mut(|v| {
                let target = &mut v.get_mut(kbd_idx).unwrap().1;
                loop {
                    match target.fetch_events() {
                        Ok(iter) => for pressed in iter {
                            println!("입력: {:#?}", pressed);
                        },
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            eprintln!("fetch_events error (kbd_idx={kbd_idx}): {}", e);
                            break;
                        }
                    }
                }
            });
        }
    }
}
