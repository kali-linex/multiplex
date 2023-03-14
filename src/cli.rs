use clap::{Parser, command};
use env_logger::{Env, Builder, fmt::{Color}};
use log::Level;
use std::{io::Write, net::{SocketAddr, IpAddr}};

pub fn init_logger() {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    Builder::from_env(env).format(|buf, record| {
        let (ch, color, bold) = match record.level() {
            Level::Error => ('X', Color::Red, true),
            Level::Warn => ('!', Color::Yellow, true),
            Level::Info => ('i', Color::Blue, true),
            Level::Debug => ('*', Color::Green, false),
            Level::Trace => ('.', Color::White, false),
        };
        let mut icon_style = buf.style();
        icon_style.set_color(color).set_bold(true);
        let mut msg_style = buf.style();
        msg_style.set_color(Color::White).set_bold(bold);


        writeln!(buf, "{} {} {}", icon_style.value(format!("[{}]", ch)), msg_style.value(buf.timestamp()), msg_style.value(record.args()))
    }).init();
}

#[derive(Debug, Parser)]
#[command(name = "multiplex")]
#[command(about = "Harass a TCP server by hooking up multiple clients to the same connection.")]
pub struct Cli {
    #[arg()]
    pub port: u16,

    #[arg(short, long, default_value = "0.0.0.0")]
    pub listen_address: IpAddr,

    #[arg()]
    pub target: SocketAddr,
}