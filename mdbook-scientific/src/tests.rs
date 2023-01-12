// make it a regular test case

// use super::*;

// const TESTCASE: &str = r###"

// # Hello there

// I link $f$ and I $x$ but `not`
// so `$nested`. $x = y$.

// ```sh
// $ foo
// bar
// baz
// ```

// $$ref:fxblck
// a = sqrt(2)
// $$

// As seen in $ref:fxblck$ yada.

// "###;

// const OUTPUT_MARKDOWN: &str = r###"

// "###;

// #[derive(Debug, Clone)]
// struct Soll {
//     lico: LiCo,
//     content: &'static str,
// }

// macro_rules! test_end2end {
// ($name:ident : $input:tt => $foo:tt ) => {
//     #[test]
//     fn $name () {
//         const LIT: &str = $input;
//         let soll: &[Soll] = &[
//             $( Soll {
//                 lico: LiCo {
//                     lineno: $lineno,
//                     column: $column,
//                 },
//                 content: $content,
//             }),*
//         ];
//         let split_tags = Vec::from_iter(dollar_split_tags_iter(LIT));
//         let s = iter_over_dollar_encompassed_blocks(LIT, split_tags)

//     }
// };
// }

// test_end2end!(basic, "a $b$ c" => "a ", "b", "c");
