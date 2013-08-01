use base::*;

// DOM Builder
pub struct ElementBuilder {
    priv stack: ~[~Element]
}

impl ElementBuilder {
    pub fn new() -> ElementBuilder {
        let e = ElementBuilder {
            stack: ~[]
        };
        e
    }

    pub fn push_event(&mut self, e: Event) -> Result<Option<Element>, ~str> {
        match e {
            PI(cont) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(PINode(cont));
                }
                Ok(None)
            }
            StartTag { name, attributes } => {
                self.stack.push(~Element {
                    name: name.clone(),
                    attributes: attributes.clone(),
                    children: ~[]
                });

                Ok(None)
            }
            EndTag { name } => {
                if self.stack.len() == 0 {
                    return Err(~"Elements not properly nested");
                }
                let elem = self.stack.pop();
                let l = self.stack.len();
                if elem.name != name {
                    Err(~"Elements not properly nested")
                } else if l == 0 {
                    Ok(Some(*elem))
                } else {
                    (*self.stack[l-1]).children.push(Element(elem));
                    Ok(None)
                }
            }
            Characters(chars) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CharacterNode(chars));
                }
                Ok(None)
            }
            CDATA(chars) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CDATANode(chars));
                }
                Ok(None)
            }
            Comment(cont) => {
                let l = self.stack.len();
                if l > 0 {
                    (*self.stack[l-1]).children.push(CommentNode(cont));
                }
                Ok(None)
            }
        }
    }
}
