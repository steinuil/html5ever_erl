mod encoders;
mod flat_sink;

use rustler::{Binary, Encoder as _, Env, NifResult, Term};
use tendril::TendrilSink;

use crate::encoders::atom;

fn parse<'a>(doc: &str) -> flat_sink::FlatSink {
    let sink = flat_sink::FlatSinkCell::new();
    let parser = html5ever::parse_document(sink, Default::default());
    let sink = parser.one(doc);
    sink
}

#[rustler::nif(schedule = "DirtyCpu")]
fn parse_html5<'a>(env: Env<'a>, document: Binary) -> NifResult<Term<'a>> {
    match str::from_utf8(document.as_slice()) {
        Ok(doc) => Ok((atom::ok(), parse(doc)).encode(env)),
        Err(_) => Err(rustler::Error::BadArg),
    }
}

fn load(_env: Env, _load_info: Term) -> bool {
    true
}

rustler::init!("html5ever_nif", load = load);
