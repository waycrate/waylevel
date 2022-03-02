use clap::{arg, Command};

pub fn set_flags() -> Command<'static> {
    let app = Command::new("waylevel")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A tool to print Wayland toplevels status and other info.")
        .arg(
            arg!(-j - -json)
                .required(false)
                .takes_value(false)
                .help("Print data as json."),
        );
    app
}
