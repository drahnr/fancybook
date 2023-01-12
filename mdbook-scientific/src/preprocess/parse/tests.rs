use super::*;
use assert_matches::assert_matches;

mod dollarsplit {
    use super::*;
    #[derive(Debug, Clone)]
    struct Soll {
        lico: LiCo,
        content: &'static str,
    }

    macro_rules! test_case {
    ($name:ident : $input:tt $( =>)? $( ($lineno:literal, $column:literal, $content:literal) ),* $(,)? ) => {
        #[test]
        fn $name () {
            const LIT: &str = $input;
            let soll: &[Soll] = &[
                $( Soll {
                    lico: LiCo {
                        lineno: $lineno,
                        column: $column,
                    },
                    content: $content,
                }),*
            ];
            let ist = Vec::from_iter(dollar_split_tags_iter(LIT));
            ist.iter().zip(soll.iter()).enumerate().for_each(|(idx, (ist, soll))| {
                // assert!(lico > previous_lico);
                dbg!((&idx, &ist, &soll));
                if idx & 0x1 == 0 {
                    assert_matches!(ist.which, Dollar::Start(s) => {
                        assert_eq!(ist.lico, soll.lico);
                        assert_eq!(s, soll.content);
                        // assert_eq!(LIT.match_indices(s).filter(|(offset, x)| offset == soll.byte_offset).count(), 1);
                    })
                } else {
                    assert_matches!(ist.which, Dollar::End(s) => {
                        assert_eq!(ist.lico, soll.lico);
                        assert_eq!(s, soll.content);
                        // assert_eq!(LIT.match_indices(s).filter(|(offset, x)| offset == soll.byte_offset).count(), 1);
                    })
                }

            })

        }
    };
}

    test_case!(bare:
    r###"a ’ c"###
    );

    test_case!(oneline:
    r###"’ $b$ c"### => (0,2, "$"), (0,4, "$")
    );

    test_case!(nonascii_oneline:
    r###" ’ℌ $b$"### => (0,4, "$"), (0,6, "$")
    );

    test_case!(oneline_unclosed:
        r###"’ $b c"### => (0,2,"$"), (0,7,"")
    );

    test_case!(dollar_block_1:
    r###"
$$
\epsilon
$$
"### => (1,1, "$$"), (3,1, "$$"));

    test_case!(pre_block_w_unclosed_inlines:
r###"
$a
<pre>
\epsilon
</pre>
$4
"### => (1,0, "$"), (1,3, ""), (5,0,"$"), (5,2, ""));

    test_case!(all_in_code_block:
r###"
```bash
$ foo $ $$ $?
```
"###
    );

    test_case!(
        iter_over_empty_intra_line_sequences: "fo’ $$_$$ bar" => (0,4,"$"),(0,5,"$"),(0,7,"$"),(0,8,"$")
    );
}

mod dollarless {
    use super::*;
    macro_rules! test_dollarless {
        ($name:ident : $input:literal => $lineno1:literal / $column1:literal .. $lineno2:literal / $column2:literal, $content:literal $(, $params:literal)? $(,)? ) => {
            #[test]
            fn $name () {
                const NAME: &str = stringify!($name);
                let is_inline = NAME.contains("inline");
                let is_block = NAME.contains("block");
                let delimiter = if is_inline && !is_block {
                    "$"
                } else if !is_inline && is_block {
                    "$$"
                } else {
                    unreachable!("Name must either include block or inline, to determine the expected delimiter!");
                };
                assert!($input.starts_with(delimiter));
                test_dollarless!(split > delimiter, $input => $lineno1 / $column1 .. $lineno2 / $column2, $content $(, $params)?);
            }
        };


        (split > $delimiter:expr, $input:literal => $lineno1:literal / $column1:literal .. $lineno2:literal / $column2:literal, $content:literal) => {
            test_dollarless!(inner > $delimiter, $input => $lineno1 / $column1 .. $lineno2 / $column2, $content, None)
        };
        (split > $delimiter:expr, $input:literal => $lineno1:literal / $column1:literal .. $lineno2:literal / $column2:literal, $content:literal, $params:literal ) => {
            test_dollarless!(inner > $delimiter, $input => $lineno1 / $column1 .. $lineno2 / $column2, $content, Some($params))
        };

        (inner > $delimiter:expr, $input:literal => $lineno1:literal / $column1:literal .. $lineno2:literal / $column2:literal, $content:literal, $maybe_params:expr ) => {
                const INPUT: &str = $input;
                let maybe_params: Option<&str> = $maybe_params;
                let (last_lineno, last_line) = INPUT.lines().enumerate().last().expect("Must have at least one line. qed");
                let (last_line_char_offset, (_, _)) = last_line.char_indices().enumerate().last().unwrap();
                let content = Content {
                    s: INPUT,
                    start: LiCo { lineno: 1, column: 1, },
                    end: LiCo { lineno: last_lineno + 1, column: last_line_char_offset, },
                    byte_range: 0..INPUT.len(),
                    start_del: Dollar::Start($delimiter),
                    end_del: Dollar::End($delimiter),
                };
                let dollarless = content.trimmed();
                assert!(dbg!(&dollarless.byte_range).len() < dbg!(&content.byte_range).len());

                assert!(dbg!(&dollarless.byte_range.start) > dbg!(&content.byte_range.start));
                assert!(dbg!(&dollarless.byte_range.end) < dbg!(&content.byte_range.end));

                assert_eq!(dollarless.trimmed, &content.s[dollarless.byte_range]);
                assert!(dbg!(dollarless.start) > dbg!(content.start));
                assert!(dbg!(dollarless.end) <= dbg!(content.end)); // FIXME must be less than!
                assert_eq!(dbg!(dollarless.parameters), dbg!(maybe_params));
                assert_eq!(dbg!(dollarless.start), dbg!(LiCo { lineno: $lineno1, column: $column1 }));
                assert_eq!(dbg!(dollarless.end), dbg!(LiCo { lineno: $lineno2, column: $column2 }));
            };
    }

    // practically not reachable, since blocks are processed first
    // test_dollarless!(empty_inline: r###"$$"### => 1/1..1/1, "");

    test_dollarless!(boring_inline: r###"$xyz$"### => 1/2..1/4, "xyz", );

    test_dollarless!(block_w_params: r###"$$params,params,params
fo’
$$"### => 2/1..2/4, r###"fo’"###, "params,params,params");

    test_dollarless!(empty_block: r###"$$
$$"### => 2/1..2/1, r###""###);

    test_dollarless!(boring_block_w_nonascii: r###"$$
hel℃⅌ charlie$
$$"### => 2/1..2/15, r###"hel℃⅌ charlie$"###);
}

mod sequester {

    use super::*;

    /// Expected value
    struct SollSequester {
        keep: bool,
        bytes: std::ops::Range<usize>,
        content: &'static str,
    }

    /// Helper constants for better legibility in test cases
    const K: bool = true;
    const R: bool = false;

    macro_rules! test_sequester {
    ($name:ident : $input:tt $( =>)? $( ($keep:ident, $byte_start:literal .. $byte_end:literal, $content:literal) ),* ) => {
            #[test]
            fn $name () {
                const LIT: &str = $input;
                let soll: &[SollSequester] = &[
                    $( SollSequester {
                        keep: $keep,
                        bytes: $byte_start .. $byte_end,
                        content: $content,
                    }),*
                ];
                let split_points_iter = dollar_split_tags_iter(LIT);
                let ist = iter_over_dollar_encompassed_blocks(LIT, split_points_iter);
                let ist = Vec::<Tagged<'_>>::from_iter(ist);
                ist.iter().zip(soll.iter()).enumerate().for_each(|(_idx, (ist, soll)): (usize, (_, &SollSequester))| {
                    assert_eq!(&LIT[soll.bytes.clone()], soll.content, "Test case integrity violated");
                    match dbg!(&ist) {
                        Tagged::Replace(_c) => { assert!(!soll.keep); }
                        Tagged::Keep(_c) => { assert!(soll.keep); }
                    }
                    let content: &Content<'_> = dbg!(ist.as_ref());
                    assert_eq!(&content.s[..], &soll.content[..]);
                    assert_eq!(&content.s[..], &LIT[soll.bytes.clone()]);
                })

            }
        };
    }

    test_sequester!(
    singlje_x:
        "x: $1$" =>
    (K, 0..3, "x: "),
    (R, 3..6, "$1$"));

    test_sequester!(
    doublje_x_y:
        "x:$1$y:$2$" =>
    (K, 0..2, "x:"),
    (R, 2..5, "$1$"),
    (K, 5..7, "y:"),
    (R, 7..10, "$2$"));

    test_sequester!(onelinje:
    "$1$ $2$ $3$" =>
(R, 0..3, "$1$"),
(K, 3..4, " "),
(R, 4..7, "$2$"),
(K, 7..8, " "),
(R, 8..11, "$3$"));

    test_sequester!(nonascii_prefix:
        "℃ $1234$" =>
    (K, 0..4, "℃ "),
    (R, 4..10, "$1234$")
    );

    test_sequester!(oneblockje:
r#"$$
1
$$"# =>
(R, 0..7, r#"$$
1
$$"#));

    test_sequester!(oneblockje_w_prefix:
    r#"Hello’ there is a block
$$
1
$$"# =>
    (K, 0..26, r#"Hello’ there is a block
"#),
    (R, 26..33, "$$
1
$$")
    );

    test_sequester!(nope:
    r####"# abc

Hello’ the block is a myth!
        
"#### =>
    (K, 0..44, r####"# abc

Hello’ the block is a myth!
        
"####)
    );
}
