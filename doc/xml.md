% Crate xml

<div class='index'>

* [Module `ElementBuilder`](#module-elementbuilder)
* [Module `Parser`](#module-parser)
* [Module `base`](#module-base)

</div>

# Module `ElementBuilder`

<div class='index'>

* [Struct `ElementBuilder`](#struct-elementbuilder) - An ELement Builder, building `Element`s from `Event`s as produced by `Parser`
* [Implementation ` for ElementBuilder`](#implementation-for-elementbuilder)

</div>

## Struct `ElementBuilder`

~~~ {.rust}
pub struct ElementBuilder {
    priv stack: ~[~Element],
}
~~~

An ELement Builder, building `Element`s from `Event`s as produced by `Parser`

## Implementation for `ElementBuilder`

### Method `new`

~~~ {.rust}
fn new() -> ElementBuilder
~~~

Returns a new `ElementBuilder`

### Method `push_event`

~~~ {.rust}
fn push_event(&mut self, e: Event) -> Result<Option<Element>, ~str>
~~~

Hands an `Event` to the builder.
While no root element has been finished `Ok(None)` is returned.
Once sufficent data has been received an `Element` is returned as `Ok(elem)`.
Upon Error `Err("message")` is returned.

# Module `Parser`

<div class='index'>

* [Struct `Parser`](#struct-parser) - A streaming XML parser
* [Implementation ` for Parser`](#implementation-for-parser)

</div>

## Struct `Parser`

~~~ {.rust}
pub struct Parser {
    priv line: uint,
    priv col: uint,
    priv buf: ~str,
    priv name: ~str,
    priv attrName: ~str,
    priv attributes: ~[Attribute],
    priv delim: char,
    priv st: State,
    priv level: uint,
}
~~~

A streaming XML parser

## Implementation for `Parser`

### Method `new`

~~~ {.rust}
fn new() -> Parser
~~~

Returns a new `Parser`

### Method `parse_str`

~~~ {.rust}
fn parse_str(&mut self, data: &str, cb: &fn(Result<Event, Error>))
~~~

Parses the string `data`.
The callback `cb` is called for each `Event`, or `Error` generated while parsing
the string.

~~~
let mut p = Parser::new();
do p.parse_str("<a href='http://rust-lang.org'>Rust</a>") |event| {
    match event {
       [...]
    }
}
~~~

# Module `base`

<div class='index'>

* [Enum `Event`](#enum-event) - Events returned by the `Parser`
* [Enum `XML`](#enum-xml) - An Enum describing a XML Node
* [Struct `Attribute`](#struct-attribute) - A struct representing an XML attribute
* [Struct `Element`](#struct-element) - A struct representing an XML element
* [Struct `Error`](#struct-error) - If an error occurs while parsing some XML, this is the structure which is  returned
* [Implementation ` of ::std::clone::Clone for XML`](#implementation-of-stdcloneclone-for-xml) - Automatically derived.
* [Implementation ` of ::std::cmp::Eq for XML`](#implementation-of-stdcmpeq-for-xml) - Automatically derived.
* [Implementation ` of ::std::clone::Clone for Element`](#implementation-of-stdcloneclone-for-element) - Automatically derived.
* [Implementation ` of ::std::cmp::Eq for Element`](#implementation-of-stdcmpeq-for-element) - Automatically derived.
* [Implementation ` of ::std::clone::Clone for Attribute`](#implementation-of-stdcloneclone-for-attribute) - Automatically derived.
* [Implementation ` of ::std::cmp::Eq for Attribute`](#implementation-of-stdcmpeq-for-attribute) - Automatically derived.
* [Implementation ` of ::std::cmp::Eq for Event`](#implementation-of-stdcmpeq-for-event) - Automatically derived.
* [Implementation ` of ::std::cmp::Eq for Error`](#implementation-of-stdcmpeq-for-error) - Automatically derived.
* [Implementation ` for XML`](#implementation-for-xml)
* [Implementation ` for Element`](#implementation-for-element)
* [Function `escape`](#function-escape) - Escapes ', ", &, <, and > with the appropriate XML entities.
* [Function `unescape`](#function-unescape) - Unescapes all valid XML entities in a string.

</div>

## Enum `Event`

Events returned by the `Parser`

#### Variants


* `PI(~str)` - Event indicating processing information was found

* `StartTag {
        name: ~str,
        attributes: ~[Attribute],
    }` - Event indicating a start tag was found

* `EndTag {
        name: ~str,
    }` - Event indicating a end tag was found

* `Characters(~str)` - Event indicating character data was found

* `CDATA(~str)` - Event indicating CDATA was found

* `Comment(~str)` - Event indicating a comment was found

## Enum `XML`

An Enum describing a XML Node

#### Variants


* `Element(~Element)` - An XML Element

* `CharacterNode(~str)` - Character Data

* `CDATANode(~str)` - CDATA

* `CommentNode(~str)` - A XML Comment

* `PINode(~str)` - Processing Information

## Struct `Attribute`

~~~ {.rust}
pub struct Attribute {
    /// The attribute's name
    name: ~str,
    /// The attribute's value
    value: ~str,
}
~~~

A struct representing an XML attribute

## Struct `Element`

~~~ {.rust}
pub struct Element {
    /// The element's name
    name: ~str,
    /// The element's `Attribute`s
    attributes: ~[Attribute],
    /// The element's child `XML` nodes
    children: ~[XML],
}
~~~

A struct representing an XML element

## Struct `Error`

~~~ {.rust}
pub struct Error {
    /// The line number at which the error occurred
    line: uint,
    /// The column number at which the error occurred
    col: uint,
    /// A message describing the type of the error
    msg: @~str,
}
~~~

If an error occurs while parsing some XML, this is the structure which is
returned

## Implementation of `::std::clone::Clone` for `XML`

Automatically derived.

### Method `clone`

~~~ {.rust}
fn clone(&self) -> XML
~~~

## Implementation of `::std::cmp::Eq` for `XML`

Automatically derived.

### Method `eq`

~~~ {.rust}
fn eq(&self, __arg_0: &XML) -> ::bool
~~~

### Method `ne`

~~~ {.rust}
fn ne(&self, __arg_0: &XML) -> ::bool
~~~

## Implementation of `::std::clone::Clone` for `Element`

Automatically derived.

### Method `clone`

~~~ {.rust}
fn clone(&self) -> Element
~~~

## Implementation of `::std::cmp::Eq` for `Element`

Automatically derived.

### Method `eq`

~~~ {.rust}
fn eq(&self, __arg_0: &Element) -> ::bool
~~~

### Method `ne`

~~~ {.rust}
fn ne(&self, __arg_0: &Element) -> ::bool
~~~

## Implementation of `::std::clone::Clone` for `Attribute`

Automatically derived.

### Method `clone`

~~~ {.rust}
fn clone(&self) -> Attribute
~~~

## Implementation of `::std::cmp::Eq` for `Attribute`

Automatically derived.

### Method `eq`

~~~ {.rust}
fn eq(&self, __arg_0: &Attribute) -> ::bool
~~~

### Method `ne`

~~~ {.rust}
fn ne(&self, __arg_0: &Attribute) -> ::bool
~~~

## Implementation of `::std::cmp::Eq` for `Event`

Automatically derived.

### Method `eq`

~~~ {.rust}
fn eq(&self, __arg_0: &Event) -> ::bool
~~~

### Method `ne`

~~~ {.rust}
fn ne(&self, __arg_0: &Event) -> ::bool
~~~

## Implementation of `::std::cmp::Eq` for `Error`

Automatically derived.

### Method `eq`

~~~ {.rust}
fn eq(&self, __arg_0: &Error) -> ::bool
~~~

### Method `ne`

~~~ {.rust}
fn ne(&self, __arg_0: &Error) -> ::bool
~~~

## Implementation for `XML`

### Method `to_str`

~~~ {.rust}
fn to_str(&self) -> ~str
~~~

Returns a string representation of the XML Node.

## Implementation for `Element`

### Method `to_str`

~~~ {.rust}
fn to_str(&self) -> ~str
~~~

Returns a string representation of the XML Element.

### Method `content_str`

~~~ {.rust}
fn content_str(&self) -> ~str
~~~

Returns the character and CDATA conatined in the element.

### Method `attribute_with_name`

~~~ {.rust}
fn attribute_with_name<'a>(&'a self, name: &str) -> Option<&'a Attribute>
~~~

Gets an `Attribute` with the specified name. When an attribute with the
specified name does not exist `None` is returned.

### Method `child_with_name`

~~~ {.rust}
fn child_with_name<'a>(&'a self, name: &str) -> Option<&'a Element>
~~~

Gets the first child `Element` with the specified name. When no child
with the specified name exists `None` is returned.

### Method `children_with_name`

~~~ {.rust}
fn children_with_name<'a>(&'a self, name: &str) -> ~[&'a Element]
~~~

Get all children `Element` with the specified name. When no child
with the specified name exists an empty vetor is returned.

## Function `escape`

~~~ {.rust}
fn escape(input: &str) -> ~str
~~~

Escapes ', ", &, <, and > with the appropriate XML entities.

## Function `unescape`

~~~ {.rust}
fn unescape(input: &str) -> ~str
~~~

Unescapes all valid XML entities in a string.

