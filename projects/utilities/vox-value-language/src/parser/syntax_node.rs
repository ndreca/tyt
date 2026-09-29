use crate::{
    BinaryOperator, ComparisonOperator, Function, LogicalOperator, NumberLiteral, UnaryOperator,
};

/// A node of the untyped syntax tree.
#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxNode {
    Binary {
        operator: BinaryOperator,

        left: Box<SyntaxNode>,

        right: Box<SyntaxNode>,
    },

    Bool(bool),

    Call {
        function: Function,

        arguments: Vec<SyntaxNode>,
    },

    Comparison {
        operator: ComparisonOperator,

        left: Box<SyntaxNode>,

        right: Box<SyntaxNode>,
    },

    Default {
        name: String,

        fallback: Box<SyntaxNode>,
    },

    Index {
        source: Box<SyntaxNode>,

        index: Box<SyntaxNode>,
    },

    Logical {
        operator: LogicalOperator,

        left: Box<SyntaxNode>,

        right: Box<SyntaxNode>,
    },

    Name(String),

    Number(NumberLiteral),

    StringLiteral(String),

    Swizzle {
        source: Box<SyntaxNode>,

        member: String,
    },

    Unary {
        operator: UnaryOperator,

        operand: Box<SyntaxNode>,
    },
}
