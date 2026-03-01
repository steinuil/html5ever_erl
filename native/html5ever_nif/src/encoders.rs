use html5ever::{QualName, tendril::StrTendril};
use rustler::Term;

use crate::flat_sink::{FlatSink, Id, Node, NodeData, Parent};

pub mod atom {
    rustler::atoms! {
        // Nil
        nil,

        // Result
        ok,
        error,

        invalid_utf8,

        // option.Option
        some,
        none,

        id,
        node,
        qualified_name,
        document,

        // NodeData
        doctype,
        text,
        comment,
        element,
        processing_instructions,
    }
}

macro_rules! tuple {
    ($env:expr, $($term:expr),*$(,)?) => {
        ::rustler::types::tuple::make_tuple($env, &[$($term.encode($env)),*])
    };
}

struct ErlQualName<'a>(&'a QualName);

impl<'b> rustler::Encoder for ErlQualName<'b> {
    fn encode<'a>(&self, env: rustler::Env<'a>) -> rustler::Term<'a> {
        self.0.local.encode(env)
    }
}

struct ErlStrTendril<'a>(&'a StrTendril);

impl<'b> rustler::Encoder for ErlStrTendril<'b> {
    fn encode<'a>(&self, env: rustler::Env<'a>) -> rustler::Term<'a> {
        self.0.encode(env)
    }
}

impl rustler::Encoder for Id {
    fn encode<'a>(&self, env: rustler::Env<'a>) -> rustler::Term<'a> {
        tuple!(env, atom::id(), self.0)
    }
}

impl rustler::Encoder for Parent {
    fn encode<'a>(&self, env: rustler::Env<'a>) -> rustler::Term<'a> {
        match self {
            Parent::Some(Id(0)) | Parent::None => atom::none().encode(env),
            Parent::Some(id) => tuple!(env, atom::some(), id),
        }
    }
}

impl rustler::Encoder for Node {
    fn encode<'a>(&self, env: rustler::Env<'a>) -> rustler::Term<'a> {
        match &self.data {
            NodeData::Document => unreachable!(),
            NodeData::Doctype {
                name,
                public_id,
                system_id,
            } => tuple!(
                env,
                atom::doctype(),
                self.id,
                self.parent,
                ErlStrTendril(name),
                ErlStrTendril(public_id),
                ErlStrTendril(system_id),
            ),
            NodeData::ProcessingInstructions { target, contents } => tuple!(
                env,
                atom::processing_instructions(),
                self.id,
                self.parent,
                ErlStrTendril(target),
                ErlStrTendril(contents),
            ),
            NodeData::Text { contents } => {
                tuple!(
                    env,
                    atom::text(),
                    self.id,
                    self.parent,
                    ErlStrTendril(&contents.borrow())
                )
            }
            NodeData::Comment { contents } => {
                tuple!(
                    env,
                    atom::comment(),
                    self.id,
                    self.parent,
                    ErlStrTendril(contents),
                )
            }
            NodeData::Element {
                name,
                attrs,
                mathml_attotation_xml_integration_point: _,
            } => {
                let attrs = rustler::Term::map_from_pairs(
                    env,
                    &attrs
                        .iter()
                        .map(|a| (ErlQualName(&a.name), ErlStrTendril(&a.value)))
                        .collect::<Vec<_>>(),
                )
                .unwrap();

                let name = tuple!(
                    env,
                    atom::qualified_name(),
                    self.id,
                    self.parent,
                    match &name.prefix {
                        None => atom::none().encode(env),
                        Some(name) => tuple!(env, atom::some(), name),
                    },
                    name.ns,
                    name.local,
                );

                tuple!(
                    env,
                    atom::element(),
                    self.id,
                    self.parent,
                    name,
                    attrs,
                    &self.children
                )
            }
        }
    }
}

impl rustler::Encoder for FlatSink {
    fn encode<'a>(&self, env: rustler::Env<'a>) -> Term<'a> {
        let (node_keys, node_values): (Vec<_>, Vec<_>) = self
            .nodes
            .iter()
            .skip(1)
            .map(|n| (n.id.0.encode(env), n.encode(env)))
            .unzip();
        let nodes_term = Term::map_from_term_arrays(env, &node_keys, &node_values).unwrap();

        let node_count = self.nodes.len();

        tuple!(
            env,
            atom::document(),
            node_count,
            self.nodes[0].children,
            nodes_term
        )
    }
}
