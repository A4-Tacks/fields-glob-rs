#![doc = include_str!("../README.md")]
use std::{collections::HashSet, convert::identity, fmt::Write};

use proc_macro::*;
use proc_macro_tool::{
    err, puncts, stream, streams, ParseIter, ParseIterExt, PunctExt, SetSpan,
    StrExt, StreamIterExt, TokenStreamExt, TokenTreeExt, WalkExt,
};

#[doc = include_str!("../README.md")]
#[proc_macro_derive(fields_glob, attributes(fields_glob_export_macro))]
pub fn fields_glob_derive(adt: TokenStream) -> TokenStream {
    let mut iter = adt.parse_iter();
    let export_macro = iter.next_attributes().into_iter().any(|attr| {
        attr.into_iter().nth(1).is_some_and(|tt| {
            tt.as_group()
                .and_then(|group| group.stream().into_iter().next())
                .is_some_and(|tt| tt.is_keyword("fields_glob_export_macro"))
        })
    });
    iter.next_vis();

    let strukt = iter.next().unwrap();
    if !strukt.is_keyword("struct") {
        return err("fields_glob only support struct", strukt);
    }

    let name = iter.next().unwrap().into_ident().unwrap();

    let mut prev = None;
    let mut last = iter.next().unwrap();

    for next in iter {
        prev = last.into();
        last = next;
    }

    let body = if !last.is_punch(';') {
        last
    } else if let Some(prev) = prev {
        prev
    } else {
        return err("fields_glob cannot support unit-like struct", last);
    }.into_group().unwrap();

    if !body.is_delimiter_brace() {
        return err("fields_glob only support named field struct", body);
    }

    let mut template = export_macro
        .then_some("#[macro_export]")
        .unwrap_or_default()
        .to_owned();
    writeln!(template, "/// `{name}! {{}}` fields_glob support macro").unwrap();
    template += r#"
    macro_rules! % {
        ($($t:tt)*) => {
            ::fields_glob::fields_glob_impl! {
                %
                @
                $($t)*
            }
        };
    }"#;
    template.parse::<TokenStream>().unwrap()
        .walk(|tt| match tt.set_spaned(name.span()) {
            tt if tt.is_punch('%') => name.clone().tt(),
            tt if tt.is_punch('@') => body.clone().tt(),
            tt => tt,
        })
}

#[proc_macro]
pub fn fields_glob_impl(input: TokenStream) -> TokenStream {
    let mut iter = input.parse_iter();
    let [name, fields] = iter.next_tts();
    let TokenTree::Ident(name) = name else {
        return err("invalid input, expected struct name", name);
    };
    let TokenTree::Group(fields) = fields else {
        return err("invalid input, expected struct body", fields);
    };
    let fields = parse_fields_declare(fields);
    parse_fields_use(name, iter, &fields)
        .unwrap_or_else(identity)
}

fn parse_fields_declare(fields: Group) -> Vec<String> {
    let mut parse_iter = fields.stream().parse_iter();
    parse_iter.next_outer_attributes();
    parse_iter
        .split_puncts_all(",")
        .filter_map(|field| {
            let mut field = field.parse_iter();
            field.next_attributes();
            field.next_vis();
            field.next()
                .and_then(|tt| tt.as_ident().map(ToString::to_string))
                .filter(|_| field.peek_is(|tt| tt.is_punch(':')))
        })
        .collect()
}

struct Star {
    attrs: Vec<TokenStream>,
    ref_tt: Option<TokenTree>,
    mut_tt: Option<TokenTree>,
    star: TokenTree,
}

fn parse_fields_use(
    name: Ident,
    mut iter: ParseIter<impl Iterator<Item = TokenTree>>,
    decl_fields: &[String],
) -> Result<TokenStream, TokenStream> {
    iter.next_outer_attributes();
    let mut star_info = None;
    let mut used_field = HashSet::new();

    let mut body = iter
        .split_puncts_all(",")
        .map(|field| {
            let mut iter = field.parse_iter();
            let attrs = iter.next_attributes();
            let ref_tt = iter.next_if(|tt| tt.is_keyword("ref"));
            let mut_tt = iter.next_if(|tt| tt.is_keyword("mut"));
            if let Some(star) = iter.next_if(|tt| tt.is_punch('*')) {
                star_info = Some(Star { attrs, ref_tt, mut_tt, star });
                Ok(None)
            } else {
                iter.next()
                    .and_then(|it| it.into_ident().ok())
                    .map(|ident| {
                        used_field.insert(ident.to_string());
                        streams(attrs.into_iter().chain([stream(
                            flat([ref_tt, mut_tt])
                                .chain([ident.tt()])
                                .chain(iter),
                        )]))
                        .into()
                    })
                    .ok_or_else(|| err!("cannot find field"))
            }
        })
        .filter_map(Result::transpose)
        .try_join(puncts(", "))?;

    if let Some(Star {
        attrs,
        ref_tt,
        mut_tt,
        star,
    }) = star_info {
        for field in decl_fields.iter()
            .filter(|field| !used_field.contains(*field))
        {
            if !body.is_empty() {
                body.push(','.alone().tt());
            }
            body.extend(attrs.iter().cloned());
            body.extend(ref_tt.clone());
            body.extend(mut_tt.clone());
            body.push(field.ident(star.span()).tt());
        }
    }

    Ok(stream([name.tt(), body.grouped_brace().tt()]))
}

fn flat<I>(iter: I) -> impl Iterator<Item = <I::Item as IntoIterator>::Item>
where I: IntoIterator,
      I::Item: IntoIterator,
{
    iter.into_iter().flatten()
}
