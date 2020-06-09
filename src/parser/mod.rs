use peg;

#[derive(Debug)]
pub enum Node {
    Program(Vec<Node>),
    Identifier(String),
    IntegerLiteral(i64),
    Block(Vec<Node>, bool),
    Call(Box<Node>, Vec<Node>),
    FunctionImplementation {
        name: String,
        params: Vec<Node>,
        return_type: Box<Node>,
        body: Box<Node>
    },
    FunctionImport {
        name: String,
        module: String,
        params: Vec<Node>,
        return_type: Box<Node>,
    },
    FunctionParameter(String, Box<Node>),
    MemSet(Box<Node>, Box<Node>)
}

use Node::*;

peg::parser!{
    pub grammar tarn_parser() for str {
        // Whitespace

        rule _() = quiet!{[' ' | '\n' | '\t']*}

        rule __() = quiet!{[' ' | '\n' | '\t']+}

        // Atoms

        rule reserved_identifier() -> ()
            = "fn" / "import"
            { () }

        rule identifier_s() -> String
            = !reserved_identifier() id:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*)
            { id.into() }

        rule identifier() -> Node
            = id:identifier_s()
            { Identifier(id) }

        rule integer_literal() -> Node
            = n:$("-"? ['0'..='9']+)
            { IntegerLiteral(n.parse().unwrap()) }

        // Types

        pub rule typ() -> Node = identifier()

        // Expressions - these cascade!

        pub rule expr() -> Node
            = mem_set()

        pub rule mem_set() -> Node
            = "set!" __ target:expr() __ value:expr()
            { MemSet(Box::new(target), Box::new(value)) }
            / block()

        rule block() -> Node
            = "{" _ stmts:expr() ** (_ ";" _) _ term:";"? _ "}"
            { Block(stmts, term.is_some()) }
            / call()
    
        rule call() -> Node 
            = target:atom() "(" _ args:expr() ** (_ "," _) _ ")"
            { Call(Box::new(target), args) }
            / atom()

        rule atom() -> Node
            = identifier() / integer_literal() / bracketed()
        
        rule bracketed() -> Node
            = "(" _ e:expr() _ ")"
            { e }

        // Top-level

        rule function_parameter() -> Node
            = id:identifier_s() _ ":" _ t:typ()
            { FunctionParameter(id, Box::new(t)) }

        // TODO: make return type optional
        pub rule function_implementation() -> Node
            = "fn" __ name:identifier_s()
              "(" _ params:function_parameter() ** ("," _) _ ")" _ "->" _ return_type:typ()
              __ body:expr()
            { FunctionImplementation { name, params, return_type: Box::new(return_type), body: Box::new(body) } }

        pub rule function_import() -> Node
            = "import" __ "fn" __ module:identifier_s() __ name:identifier_s()
              "(" _ params:function_parameter() ** ("," _) _ ")" _ "->" _ return_type:typ() _ ";"
            { FunctionImport { module, name, params, return_type: Box::new(return_type) } }

        pub rule program() -> Node
            = ";"* _ n:(function_import() / function_implementation()) ** (_ ";"* _) ";"* _
            { Program(n) }
    }
}