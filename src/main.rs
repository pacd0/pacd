use anyhow::Result;
use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use clap::{App, Arg};
use handlebars::Handlebars;
use log::*;
use serde_json::json;
use std::env;
use tide::{middleware::CRequest, middleware::RequestLogger, Response, Server};

async fn get_domains(file: &str) -> Result<Vec<String>> {
    let f = File::open(file).await?;
    let reader = BufReader::new(f);
    let mut domains = vec![];
    let mut lines = reader.lines();
    while let Some(line) = lines.next().await {
        domains.push(line?);
    }
    Ok(domains)
}

async fn get_proxy_pac(domains: Vec<String>) -> Result<String> {
    let template = include_str!("../data/proxy.pac.hbs");
    debug!("domains: {:?}", domains);
    let reg = Handlebars::new();
    let result = reg.render_template(template, &json!({ "domains": domains }))?;
    Ok(result)
}

pub struct State {
    proxy_pac: String,
}

impl State {
    pub fn proxy_pac(&self) -> &str {
        self.proxy_pac.as_str()
    }
}

async fn handle_proxy_pac(req: Request<State>) -> Response {
    let proxy_pac = req.state().proxy_pac().into();
    Response::new(200).body_string(proxy_pac).set_header(
        "content-type",
        "application/x-ns-proxy-autoconfig;charset=utf-8",
    )
}

#[async_std::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "pacd=debug,tide=debug");
    dotenv::dotenv().ok();
    femme::start(log::LevelFilter::Trace)?;

    let matches = App::new("PAC Daemon")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("listen")
                .long("listen")
                .short("l")
                .default_value("127.0.0.1:8080")
                .help("Sets bind address"),
        )
        .arg(
            Arg::with_name("file")
                .required(true)
                .index(1)
                .help("Domain list file"),
        )
        .get_matches();

    match matches.occurrences_of("verbose") {
        0 => log::set_max_level(log::LevelFilter::Warn),
        1 => log::set_max_level(log::LevelFilter::Info),
        _ => log::set_max_level(log::LevelFilter::Debug),
    }

    let file = matches.value_of("file").unwrap();
    let domains = get_domains(file).await?;
    let proxy_pac = get_proxy_pac(domains).await?;

    let addr = matches.value_of("listen").unwrap();
    info!("listening at: {}", addr);

    let mut app = Server::with_state(State { proxy_pac });
    app.middleware(RequestLogger::new());
    app.at("/").get(|_| async move { "Hello, world!" });
    app.at("/proxy.pac").get(handle_proxy_pac);
    app.listen(addr).await?;
    Ok(())
}
