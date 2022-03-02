mod clap;
mod toplevel;
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
struct ToplevelHandleDataJSON {
    title: String,
    app_id: String,
    state: toplevel::ToplevelHandleStates,
}

fn main() -> () {
    let args = clap::set_flags().get_matches();
    let toplevels = match toplevel::get_toplevel_data() {
        Ok(toplevel) => toplevel,
        Err(e) => {
            println!("Err: {:#?}", e);
            exit(1);
        }
    };
    if args.is_present("json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&populate_json_struct(toplevels)).unwrap()
        );
    } else {
        print_human_readable(toplevels);
    }
}

fn print_human_readable(toplevels: Vec<toplevel::ToplevelHandleData>) {
    let mut count = 0;
    for toplevel in toplevels {
        let mut state = String::new();
        match toplevel.state.is_maximized {
            true => state = state + "m",
            false => state = state + "-",
        };
        match toplevel.state.is_minimized {
            true => state = state + "m",
            false => state = state + "-",
        };
        match toplevel.state.is_activated {
            true => state = state + "a",
            false => state = state + "-",
        };
        match toplevel.state.is_fullscreen {
            true => state = state + "f",
            false => state = state + "-",
        };
        println!(
            "{}: {} \"{}\" {}",
            count, state, toplevel.title, toplevel.app_id
        );
        count += 1;
    }
}

fn populate_json_struct(
    toplevels: Vec<toplevel::ToplevelHandleData>,
) -> Vec<ToplevelHandleDataJSON> {
    let mut json_toplevel: Vec<ToplevelHandleDataJSON> = Vec::new();
    for toplevel in toplevels {
        json_toplevel.push(ToplevelHandleDataJSON {
            title: toplevel.title,
            app_id: toplevel.app_id,
            state: toplevel.state,
        });
    }
    json_toplevel
}
