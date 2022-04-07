mod clap;
mod output;
mod toplevel;
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::reexports::client::{Display, GlobalManager};
use std::process::exit;

// We recreate each struct and cherry pick the data because not all the fields
// can be denoted as valid JSON via serde_json.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
struct ToplevelHandleDataJSON {
    title: String,
    app_id: String,
    state: toplevel::ToplevelHandleStates,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
struct OutputModeJSON {
    dimensions: (i32, i32),
    refresh_rate: String,
    is_current: bool,
    is_preferred: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
struct OutputInfoJSON {
    id: u32,
    model: String,
    make: String,
    name: String,
    description: String,
    scale_factor: i32,
    modes: Vec<OutputModeJSON>,
    is_obsolete: bool,
}

fn main() {
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
    } else if args.is_present("globals") {
        print_compositor_globals();
    } else if args.is_present("outputs") {
        println!(
            "{}",
            serde_json::to_string_pretty(&print_output_info()).unwrap()
        );
    } else {
        print_human_readable(toplevels);
    }
}

fn print_human_readable(toplevels: Vec<toplevel::ToplevelHandleData>) {
    for (count, toplevel) in toplevels.into_iter().enumerate() {
        let mut state = String::new();
        match toplevel.state.is_maximized {
            true => state += "m",
            false => state += "-",
        };
        match toplevel.state.is_minimized {
            true => state += "m",
            false => state += "-",
        };
        match toplevel.state.is_activated {
            true => state += "a",
            false => state += "-",
        };
        match toplevel.state.is_fullscreen {
            true => state += "f",
            false => state += "-",
        };
        println!(
            "{}: {} \"{}\" {}",
            count, state, toplevel.title, toplevel.app_id
        );
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

fn print_compositor_globals() {
    let display = Display::connect_to_env().unwrap();
    let mut event_queue = display.create_event_queue();
    let attached_display = (*display).clone().attach(event_queue.token());

    let globals = GlobalManager::new(&attached_display);

    event_queue
        .sync_roundtrip(&mut (), |_, _, _| unreachable!())
        .unwrap();

    for (id, interface, version) in globals.list() {
        println!("{}: {} (version {})", id, interface, version);
    }
}

fn print_output_info() -> Vec<OutputInfoJSON> {
    let display = Display::connect_to_env().unwrap();
    let outputs = output::get_valid_outputs(display);
    let mut output_info: Vec<OutputInfoJSON> = Vec::new();
    for (_, info) in outputs {
        let mut modes: Vec<OutputModeJSON> = Vec::new();
        for mode in info.modes {
            let refresh_rate = (mode.refresh_rate / 1000).to_string() + " Hz";
            modes.push(OutputModeJSON {
                dimensions: mode.dimensions,
                refresh_rate,
                is_current: mode.is_current,
                is_preferred: mode.is_preferred,
            });
        }
        output_info.push(OutputInfoJSON {
            id: info.id,
            model: info.model,
            make: info.make,
            name: info.name,
            description: info.description,
            scale_factor: info.scale_factor,
            modes,
            is_obsolete: info.obsolete,
        });
    }
    output_info
}
