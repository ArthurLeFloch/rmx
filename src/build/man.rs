use rmx::arguments::Args;

use clap::CommandFactory;
use clap_mangen::Man;
use std::io;

fn main() -> io::Result<()> {
    let cmd = Args::command();
    let man = Man::new(cmd);
    man.render(&mut io::stdout())
}
