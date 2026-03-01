use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
};

use html5ever::{
    Attribute, QualName,
    interface::{NodeOrText, TreeSink},
    tendril::StrTendril,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Id(pub usize);

impl Id {
    pub fn document() -> Self {
        Id(0)
    }
}

#[derive(Debug)]
pub enum Parent {
    Some(Id),
    None,
}

#[derive(Debug)]
pub enum NodeData {
    Document,
    Doctype {
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    },
    Text {
        contents: RefCell<StrTendril>,
    },
    Comment {
        contents: StrTendril,
    },
    Element {
        name: QualName,
        attrs: Vec<Attribute>,
        mathml_attotation_xml_integration_point: bool,
    },
    ProcessingInstructions {
        target: StrTendril,
        contents: StrTendril,
    },
}

#[derive(Debug)]
pub struct Node {
    pub id: Id,
    pub parent: Parent,
    pub children: Vec<Id>,
    pub data: NodeData,
}

impl Node {
    fn new(id: Id, data: NodeData) -> Node {
        Node {
            id,
            parent: Parent::None,
            children: vec![],
            data,
        }
    }

    fn index_of_child(&self, child: Id) -> Option<usize> {
        self.children.iter().position(|&x| x == child)
    }

    fn has_parent(&self) -> bool {
        match self.parent {
            Parent::Some(_) => true,
            Parent::None => false,
        }
    }
}

#[derive(Debug)]
pub struct FlatSink {
    pub nodes: Vec<Node>,
}

impl FlatSink {
    fn create_node(&mut self, data: NodeData) -> Id {
        let id = Id(self.nodes.len());
        self.nodes.push(Node::new(id, data));
        id
    }

    fn append_node(&mut self, parent: Id, child: Id) {
        self.nodes[child.0].parent = Parent::Some(parent);
        self.nodes[parent.0].children.push(child);
    }

    fn append_text(&mut self, parent: Id, text: StrTendril) {
        match self.nodes[parent.0]
            .children
            .last()
            .map(|last_id| &self.nodes[last_id.0])
        {
            Some(Node {
                data: NodeData::Text { contents },
                ..
            }) => {
                contents.borrow_mut().push_tendril(&text);
            }
            Some(_) | None => {
                let child = self.create_node(NodeData::Text {
                    contents: RefCell::new(text),
                });

                self.append_node(parent, child);
            }
        }
    }

    fn get_parent_and_index(&self, child: Id) -> Option<(Id, usize)> {
        match self.nodes[child.0].parent {
            Parent::None => None,
            Parent::Some(parent) => match self.nodes[parent.0].index_of_child(child) {
                Some(i) => Some((parent, i)),
                None => unreachable!("have parent but not in parent"),
            },
        }
    }

    fn get_template_contents(&self, target: Id) -> Id {
        match self.nodes[target.0].data {
            NodeData::Element { .. } => target,
            _ => unreachable!("not a template element"),
        }
    }

    fn remove_from_parent(&mut self, target: Id) {
        if let Some((parent, i)) = self.get_parent_and_index(target) {
            self.nodes[parent.0].children.remove(i);
            self.nodes[target.0].parent = Parent::None;
        }
    }

    fn append_before_sibling(&mut self, sibling: Id, new_node: NodeOrText<Id>) {
        let (parent, i) = self
            .get_parent_and_index(sibling)
            .expect("append_before_sibling called on node without parent");

        let child = match (new_node, i) {
            (NodeOrText::AppendText(text), 0) => self.create_node(NodeData::Text {
                contents: RefCell::new(text),
            }),
            (NodeOrText::AppendText(text), i) => {
                let prev = self.nodes[parent.0].children[i - 1];
                match &self.nodes[prev.0].data {
                    NodeData::Text { contents } => {
                        contents.borrow_mut().push_tendril(&text);
                        return;
                    }
                    _ => self.create_node(NodeData::Text {
                        contents: RefCell::new(text),
                    }),
                }
            }
            (NodeOrText::AppendNode(node), _) => node,
        };

        if self.nodes[child.0].has_parent() {
            self.remove_from_parent(child);
        }

        self.nodes[child.0].parent = Parent::Some(parent);
        self.nodes[parent.0].children.insert(i, child);
    }
}

pub struct FlatSinkCell(RefCell<FlatSink>);

impl FlatSinkCell {
    pub fn new() -> Self {
        Self(RefCell::new(FlatSink {
            nodes: vec![Node::new(Id(0), NodeData::Document)],
        }))
    }
}

impl TreeSink for FlatSinkCell {
    type Handle = Id;

    type Output = FlatSink;

    type ElemName<'a>
        = Ref<'a, QualName>
    where
        Self: 'a;

    fn finish(self) -> Self::Output {
        self.0.into_inner()
    }

    fn parse_error(&self, _msg: std::borrow::Cow<'static, str>) {}

    fn set_quirks_mode(&self, _mode: html5ever::interface::QuirksMode) {}

    fn get_document(&self) -> Self::Handle {
        Id::document()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        Ref::map(self.0.borrow(), |sink| match &sink.nodes[target.0].data {
            NodeData::Element { name, .. } => name,
            _ => unreachable!("not a template element"),
        })
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        let sink = self.0.borrow();
        sink.get_template_contents(*target)
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        self.0.borrow_mut().create_node(NodeData::Element {
            name,
            attrs,
            mathml_attotation_xml_integration_point: flags.mathml_annotation_xml_integration_point,
        })
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        self.0
            .borrow_mut()
            .create_node(NodeData::Comment { contents: text })
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        self.0
            .borrow_mut()
            .create_node(NodeData::ProcessingInstructions {
                target,
                contents: data,
            })
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let mut sink = self.0.borrow_mut();

        match child {
            NodeOrText::AppendNode(node) => sink.append_node(*parent, node),
            NodeOrText::AppendText(text) => sink.append_text(*parent, text),
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let mut sink = self.0.borrow_mut();

        let has_parent = sink.nodes[element.0].has_parent();

        if has_parent {
            sink.append_before_sibling(*element, child);
        } else {
            match child {
                NodeOrText::AppendNode(node) => sink.append_node(*prev_element, node),
                NodeOrText::AppendText(text) => sink.append_text(*prev_element, text),
            }
        }
    }

    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let mut sink = self.0.borrow_mut();
        let doctype = sink.create_node(NodeData::Doctype {
            name,
            public_id,
            system_id,
        });
        sink.append_node(Id(0), doctype);
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        self.0
            .borrow_mut()
            .append_before_sibling(*sibling, new_node);
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let mut sink = self.0.borrow_mut();

        let target_attrs = match sink.nodes[target.0].data {
            NodeData::Element { ref mut attrs, .. } => attrs,
            _ => panic!("not an element"),
        };

        let existing_names = target_attrs
            .iter()
            .map(|e| e.name.clone())
            .collect::<HashSet<_>>();
        target_attrs.extend(
            attrs
                .into_iter()
                .filter(|attr| !existing_names.contains(&attr.name)),
        )
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.0.borrow_mut().remove_from_parent(*target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut sink = self.0.borrow_mut();

        for child in sink.nodes[node.0].children.clone() {
            sink.remove_from_parent(child);
            sink.append_node(*new_parent, child);
        }
    }
}
