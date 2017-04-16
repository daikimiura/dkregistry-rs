extern crate dkregistry;
extern crate tokio_core;
extern crate futures;
extern crate log;
extern crate env_logger;

use std::{error, boxed};
use tokio_core::reactor::Core;

type Result<T> = std::result::Result<T, boxed::Box<error::Error>>;

fn main() {
    let registry = match std::env::args().nth(1) {
        Some(x) => x,
        None => "quay.io".into(),
    };

    let image = match std::env::args().nth(2) {
        Some(x) => x,
        None => "coreos/etcd".into(),
    };

    let ver = match std::env::args().nth(3) {
        Some(x) => x,
        None => "latest".into(),
    };
    println!("[{}] downloading image {} version {}", registry, image, ver);

    let user = std::env::var("DKREG_USER").ok();
    if user.is_none() {
        println!("[{}] no $DKREG_USER for login user", registry);
    }
    let password = std::env::var("DKREG_PASSWD").ok();
    if password.is_none() {
        println!("[{}] no $DKREG_PASSWD for login password", registry);
    }


    let res = run(&registry, user, password, &image, &ver);

    if let Err(e) = res {
        println!("[{}] {:?}", registry, e);
        std::process::exit(1);
    };
}

fn run(host: &str,
       user: Option<String>,
       passwd: Option<String>,
       image: &str,
       version: &str)
       -> Result<()> {
    env_logger::LogBuilder::new()
        .filter(Some("dkregistry"), log::LogLevelFilter::Trace)
        .filter(Some("trace"), log::LogLevelFilter::Trace)
        .init()?;

    let mut tcore = try!(Core::new());
    let mut dclient = try!(dkregistry::v2::Client::configure(&tcore.handle())
                               .registry(host)
                               .insecure_registry(false)
                               .username(user)
                               .password(passwd)
                               .build());

    let futcheck = try!(dclient.is_v2_supported());
    let supported = try!(tcore.run(futcheck));
    if !supported {
        return Err("API v2 not supported".into());
    }

    let fut_token = try!(dclient.login(vec![&format!("repository:{}:pull", image)]));
    let token_auth = try!(tcore.run(fut_token));

    let futauth = try!(dclient.is_auth(Some(token_auth.token())));
    if !try!(tcore.run(futauth)) {
        return Err("login failed".into());
    }

    dclient.set_token(Some(token_auth.token()));

    let fut_manif = dclient.get_manifest(image, version)?;
    let manifest = tcore.run(fut_manif)?;
    let layers = manifest.get_layers();

    for digest in layers {
        let fut_presence = dclient.has_blob(image, &digest)?;
        let has_blob = tcore.run(fut_presence)?;
        if !has_blob {
            return Err(format!("missing layer {}", digest).into());
        }

        let fut_out = dclient.get_blob(image, &digest)?;
        let _out = tcore.run(fut_out)?;
    }

    return Ok(());
}