use smithay_client_toolkit::reexports::{
    client::{Display, GlobalManager, Main},
    protocols::wlr::unstable::foreign_toplevel::v1::client::{
        zwlr_foreign_toplevel_handle_v1,
        zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
        zwlr_foreign_toplevel_manager_v1,
        zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
    },
};

use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

fn main() -> Result<(), Box<dyn Error>> {
    let display = Display::connect_to_env().unwrap();
    let mut event_queue = display.create_event_queue();
    let attached_display = display.attach(event_queue.token());
    let globals = GlobalManager::new(&attached_display);

    let toplevel_finished = Rc::new(AtomicBool::new(false));
    let toplevel_handles: Rc<RefCell<Vec<Main<ZwlrForeignToplevelHandleV1>>>> =
        Rc::new(RefCell::new(Vec::new()));

    event_queue
        .dispatch(&mut (), |_, _, _| unreachable!())
        .unwrap();

    let foreign_toplevel_manager = globals.instantiate_exact::<ZwlrForeignToplevelManagerV1>(3);

    foreign_toplevel_manager.as_ref().unwrap().quick_assign({
        let toplevel_finished = toplevel_finished.clone();
        let toplevel_handles = toplevel_handles.clone();
        move |_manager, event, _| match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                toplevel_handles.borrow_mut().push(toplevel.clone());
                toplevel.quick_assign({
                    move |_toplevel, event, _| match event {
                        zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                            println!("Title event: {}", title);
                        }
                        zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                            println!("AppId event: {}", app_id);
                        }
                        zwlr_foreign_toplevel_handle_v1::Event::OutputEnter { .. } => {}
                        zwlr_foreign_toplevel_handle_v1::Event::OutputLeave { .. } => {}
                        zwlr_foreign_toplevel_handle_v1::Event::State { state } => {
                            // state => [is_maximized, is_minimized, is_activated, is_fullscreen]
                            //println!("is maximized ? {}\nis minimized ? {}\nis_activated ? {}\nis_fullscreen ? {}", state[0], state[1], state[2], state[3]);
                            println!("State data: {:#?}", state);

                            //  use the first parameter of the closure as a key to check.
                        }
                        zwlr_foreign_toplevel_handle_v1::Event::Done { .. } => {}
                        zwlr_foreign_toplevel_handle_v1::Event::Closed { .. } => {}
                        zwlr_foreign_toplevel_handle_v1::Event::Parent { .. } => {}
                        _ => unreachable!(),
                    }
                })
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished { .. } => {
                println!("Finished event triggered!");
                toplevel_finished.store(true, Ordering::SeqCst);
            }
            _ => {
                unreachable!()
            }
        }
    });

    while !toplevel_finished.load(Ordering::SeqCst) {
        event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!())?;
        foreign_toplevel_manager.as_ref().unwrap().stop();
    }

    println!("Recieved all events");
    Ok(())
}
