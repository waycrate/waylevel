use smithay_client_toolkit::reexports::{
    client::{Display, GlobalManager},
    protocols::wlr::unstable::foreign_toplevel::v1::client::{
        zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_handle_v1::State,
        zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
        zwlr_foreign_toplevel_manager_v1,
        zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
    },
};

use serde::{Deserialize, Serialize};

use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ToplevelHandleStates {
    pub is_maximized: bool,
    pub is_minimized: bool,
    pub is_activated: bool,
    pub is_fullscreen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToplevelHandleData {
    pub handle: ZwlrForeignToplevelHandleV1,
    pub title: String,
    pub app_id: String,
    pub state: ToplevelHandleStates,
}

pub fn get_toplevel_data() -> Result<Vec<ToplevelHandleData>, Box<dyn Error>> {
    let display = Display::connect_to_env().unwrap();
    let mut event_queue = display.create_event_queue();
    let attached_display = display.attach(event_queue.token());
    let globals = GlobalManager::new(&attached_display);

    let toplevel_finished = Rc::new(AtomicBool::new(false));
    let toplevel_handle_data: Rc<RefCell<Vec<ToplevelHandleData>>> =
        Rc::new(RefCell::new(Vec::new()));

    event_queue
        .dispatch(&mut (), |_, _, _| unreachable!())
        .unwrap();

    let foreign_toplevel_manager = globals.instantiate_exact::<ZwlrForeignToplevelManagerV1>(3);

    foreign_toplevel_manager.as_ref().unwrap().quick_assign({
        let toplevel_finished = toplevel_finished.clone();
        let toplevel_handle_data = toplevel_handle_data.clone();

        move |_manager, event, _| match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => toplevel
                .quick_assign({
                    let toplevel_handle_data = toplevel_handle_data.to_owned();
                    move |toplevel, event, _| match event {
                        zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                            let mut is_present: bool = false; // this implementation is really messy and being repeated way too many times, but I don't know how to fix this right now, someone needs to clean this up.
                            let arr = toplevel_handle_data.borrow_mut().to_vec();
                            for _ in 0..arr.len() {
                                let mut element = toplevel_handle_data.borrow_mut().pop().unwrap();
                                if element.handle == toplevel.clone().detach() {
                                    element.title = title.clone();
                                    is_present = true;
                                }
                                toplevel_handle_data.borrow_mut().push(element);
                            }
                            if !is_present {
                                let toplevel_data = ToplevelHandleData {
                                    handle: toplevel.detach(),
                                    title,
                                    app_id: String::from("None"),
                                    state: ToplevelHandleStates::default(),
                                };
                                toplevel_handle_data.borrow_mut().push(toplevel_data);
                            }
                        }
                        zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                            let mut is_present: bool = false;
                            let arr = toplevel_handle_data.borrow_mut().to_vec();
                            for _ in 0..arr.len() {
                                let mut element = toplevel_handle_data.borrow_mut().pop().unwrap();
                                if element.handle == toplevel.clone().detach() {
                                    element.app_id = app_id.clone();
                                    is_present = true;
                                }
                                toplevel_handle_data.borrow_mut().push(element);
                            }
                            if !is_present {
                                let toplevel_data = ToplevelHandleData {
                                    handle: toplevel.detach(),
                                    title: String::from("None"),
                                    app_id,
                                    state: ToplevelHandleStates::default(),
                                };
                                toplevel_handle_data.borrow_mut().push(toplevel_data);
                            }
                        }
                        zwlr_foreign_toplevel_handle_v1::Event::OutputEnter { .. } => {}
                        zwlr_foreign_toplevel_handle_v1::Event::OutputLeave { .. } => {}
                        zwlr_foreign_toplevel_handle_v1::Event::State { state } => {
                            let mut state_data = ToplevelHandleStates::default();
                            let mut is_present: bool = false;
                            let arr = toplevel_handle_data.borrow_mut().to_vec();
                            let translated_states = state
                                .chunks_exact(4)
                                .map(|c| u32::from_ne_bytes(c.try_into().unwrap()))
                                .flat_map(State::from_raw)
                                .collect::<Vec<_>>();

                            for state in translated_states {
                                match state {
                                    State::Maximized => {
                                        state_data.is_maximized = true;
                                    }
                                    State::Minimized => {
                                        state_data.is_minimized = true;
                                    }
                                    State::Activated => {
                                        state_data.is_activated = true;
                                    }
                                    State::Fullscreen => {
                                        state_data.is_fullscreen = true;
                                    }
                                    _ => unreachable!(),
                                };
                            }

                            for _ in 0..arr.len() {
                                let mut element = toplevel_handle_data.borrow_mut().pop().unwrap();
                                if element.handle == toplevel.clone().detach() {
                                    element.state = state_data.clone();
                                    is_present = true;
                                }
                                toplevel_handle_data.borrow_mut().push(element);
                            }
                            if !is_present {
                                let toplevel_data = ToplevelHandleData {
                                    handle: toplevel.detach(),
                                    title: String::from("None"),
                                    app_id: String::from("None"),
                                    state: state_data,
                                };
                                toplevel_handle_data.borrow_mut().push(toplevel_data);
                            }
                        }
                        zwlr_foreign_toplevel_handle_v1::Event::Done { .. } => {}
                        zwlr_foreign_toplevel_handle_v1::Event::Closed { .. } => {}
                        zwlr_foreign_toplevel_handle_v1::Event::Parent { .. } => {}
                        _ => unreachable!(),
                    }
                }),
            zwlr_foreign_toplevel_manager_v1::Event::Finished { .. } => {
                toplevel_finished.store(true, Ordering::SeqCst);
            }
            _ => unreachable!(),
        }
    });

    if !toplevel_finished.load(Ordering::SeqCst) {
        event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!())?;
        foreign_toplevel_manager.as_ref().unwrap().stop();
    }
    return Ok(toplevel_handle_data.borrow_mut().to_vec());
}
