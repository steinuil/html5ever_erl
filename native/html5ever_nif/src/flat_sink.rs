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
}

#[derive(Debug)]
pub struct FlatSink {
    pub nodes: RefCell<Vec<Node>>,
}

impl FlatSink {
    pub fn new() -> Self {
        FlatSink {
            nodes: RefCell::new(vec![Node::new(Id(0), NodeData::Document)]),
        }
    }
}

impl FlatSink {
    fn add_node(&self, data: NodeData) -> Id {
        let id = Id(self.nodes.borrow().len());
        self.nodes.borrow_mut().push(Node::new(id, data));
        id
    }

    fn append_node(&self, parent: Id, child: Id) {
        let mut nodes = self.nodes.borrow_mut();

        nodes[child.0].parent = Parent::Some(parent);

        let parent_node = &mut nodes[parent.0];
        parent_node.children.push(child);
    }

    fn append_text(&self, parent: Id, text: StrTendril) {
        let nodes = self.nodes.borrow();

        match nodes[parent.0]
            .children
            .last()
            .map(|last_id| &nodes[last_id.0])
        {
            Some(Node {
                data: NodeData::Text { contents },
                ..
            }) => {
                contents.borrow_mut().push_tendril(&text);
            }
            Some(_) | None => {
                drop(nodes);

                let child = self.add_node(NodeData::Text {
                    contents: RefCell::new(text),
                });

                self.append_node(parent, child);
            }
        }
    }

    fn get_parent_and_index(&self, child: Id) -> Option<(Id, usize)> {
        let nodes = self.nodes.borrow();
        match nodes[child.0].parent {
            Parent::None => None,
            Parent::Some(parent) => match nodes[parent.0].index_of_child(child) {
                Some(i) => Some((parent, i)),
                None => unreachable!("have parent but not in parent"),
            },
        }
    }
}

impl TreeSink for FlatSink {
    type Handle = Id;

    type Output = Self;

    type ElemName<'a>
        = Ref<'a, QualName>
    where
        Self: 'a;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&self, _msg: std::borrow::Cow<'static, str>) {}

    fn set_quirks_mode(&self, _mode: html5ever::interface::QuirksMode) {}

    fn get_document(&self) -> Self::Handle {
        Id(0)
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        Ref::map(self.nodes.borrow(), |nodes| match nodes[target.0].data {
            NodeData::Element { ref name, .. } => name,
            _ => unreachable!("not an element"),
        })
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        match self.nodes.borrow()[target.0].data {
            NodeData::Element { .. } => *target,
            _ => unreachable!("not a template element"),
        }
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        self.add_node(NodeData::Element {
            name,
            attrs,
            mathml_attotation_xml_integration_point: flags.mathml_annotation_xml_integration_point,
        })
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        self.add_node(NodeData::Comment { contents: text })
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        self.add_node(NodeData::ProcessingInstructions {
            target,
            contents: data,
        })
    }

    fn append(&self, parent: &Self::Handle, child: html5ever::interface::NodeOrText<Self::Handle>) {
        match child {
            html5ever::interface::NodeOrText::AppendNode(node) => self.append_node(*parent, node),
            html5ever::interface::NodeOrText::AppendText(text) => self.append_text(*parent, text),
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        match &self.nodes.borrow()[element.0].parent {
            Parent::Some(_) => {
                self.append_before_sibling(element, child);
            }
            Parent::None => {
                self.append(prev_element, child);
            }
        };
    }

    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let doctype = self.add_node(NodeData::Doctype {
            name,
            public_id,
            system_id,
        });
        self.append_node(Id(0), doctype);
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn append_before_sibling(
        &self,
        sibling: &Self::Handle,
        new_node: html5ever::interface::NodeOrText<Self::Handle>,
    ) {
        let (parent, i) = self
            .get_parent_and_index(*sibling)
            .expect("append_before_sibling called on node without parent");

        let child = match (new_node, i) {
            (NodeOrText::AppendText(text), 0) => self.add_node(NodeData::Text {
                contents: RefCell::new(text),
            }),
            (NodeOrText::AppendText(text), i) => {
                let nodes = self.nodes.borrow();
                let prev = nodes[parent.0].children[i - 1];
                match &nodes[prev.0].data {
                    NodeData::Text { contents } => {
                        contents.borrow_mut().push_tendril(&text);
                        return;
                    }
                    _ => self.add_node(NodeData::Text {
                        contents: RefCell::new(text),
                    }),
                }
            }
            (NodeOrText::AppendNode(node), _) => node,
        };

        match self.nodes.borrow()[child.0].parent {
            Parent::None => {}
            Parent::Some(_) => {
                self.remove_from_parent(&child);
            }
        }

        let mut nodes = self.nodes.borrow_mut();
        nodes[child.0].parent = Parent::Some(parent);
        nodes[parent.0].children.insert(i, child);
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let mut nodes = self.nodes.borrow_mut();

        let target_attrs = match nodes[target.0].data {
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
        if let Some((parent, i)) = self.get_parent_and_index(*target) {
            let mut nodes = self.nodes.borrow_mut();
            nodes[parent.0].children.remove(i);
            let child = &mut nodes[target.0];
            child.parent = Parent::None;
        }
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        for child in &self.nodes.borrow()[node.0].children.clone() {
            self.remove_from_parent(child);
            self.append_node(*new_parent, *child);
        }
    }
}
