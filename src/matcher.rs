use std::collections::{HashMap};
use crate::parser::{Quantifier, RE};

#[derive(Debug)]
struct Additional<'a> {
    origin: &'a str,
    group_refs: HashMap<usize, Vec<&'a str>>,
}

pub fn r#match(target: &str, re: &RE) -> bool {
    let mut iter = target.chars().peekable();
    while iter.peek().is_some() {
        if match_re(vec![&iter.clone().collect::<String>()], re, &mut Additional {
            origin: target,
            group_refs: HashMap::new()
        }).is_some() {
            return true;
        }
        iter.next();
    }

    false
}

fn match_re<'a>(inputs: Vec<&'a str>, re: &RE, additional: &mut Additional<'a>) -> Option<Vec<&'a str>>{
    println!("inputs {inputs:?} \n re {re:?} \n {additional:?} \n \n");
    
    let next_inputs = inputs
        .iter()
        .filter_map(|&input| {
            match re {
                RE::Seq(seq) => {
                    let mut inputs_slice = vec![input];
                    for re in seq {
                        if let Some(next_inputs) = match_re(inputs_slice.clone(), re, additional) {
                            inputs_slice = next_inputs;
                        } else {
                            return None;
                        }
                    }
                    Some(inputs_slice)
                }
                RE::AnchorStart if input.len() == additional.origin.len() => {
                    Some(vec![input])
                }
                RE::AnchorEnd if input.is_empty() => {
                    Some(vec![input])
                }
                RE::Dot => {
                    let mut chars = input.chars();
                    if chars.next().is_some() {
                        Some(vec![chars.as_str()])
                    } else {
                        None
                    }
                }
                RE::Repeat(re, q) => {
                    let mut next_inputs: Vec<&str> = vec![];
                    if matches!(q, Quantifier::ZeroOrMore | Quantifier::ZeroOrOne) {
                        next_inputs.push(input);
                    }
                    if matches!(q, Quantifier::ZeroOrOne | Quantifier::OneOrMore) {
                        if let Some(another_next_inputs) = match_re(vec![input], re, additional) {
                            next_inputs.extend(another_next_inputs);
                        }
                    }
                    if matches!(q, Quantifier::ZeroOrMore | Quantifier::OneOrMore) {
                        if let Some(inputs_slice) = match_re(vec![input], re, additional) {
                            let mut inputs_slice = inputs_slice;
                            while let Some(input) = inputs_slice.pop() {
                                if let Some(next_inputs_slice) = match_re(vec![input], re, additional) {
                                    next_inputs.extend(&next_inputs_slice);
                                    inputs_slice.extend(&next_inputs_slice);
                                }
                            }
                        }
                    }
                    if next_inputs.is_empty() { None } else { Some(next_inputs) }
                }
                RE::Word => {
                    let mut iter = input.chars();
                    if let Some(c) = iter.next() {
                        if c.is_ascii_alphanumeric() || c == '_' {
                            Some(vec![iter.as_str()])
                        } else { None }
                    } else { None }
                }
                RE::Digit => {
                    let mut iter = input.chars();
                    if let Some(c) = iter.next() {
                        if c.is_ascii_digit() {
                            Some(vec![iter.as_str()])
                        } else { None }
                    } else { None }
                }
                RE::BackRef(index) => {
                    if let Some(refs) = additional.group_refs.get(index) {
                        let res = refs.iter()
                            .filter_map(|&r| {
                                if input.starts_with(r) {
                                    Some(&input[r.len()..])
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<&str>>();
                        if res.is_empty() { None } else { Some(res) }
                    } else {
                        None
                    }
                }
                RE::CharClass(char_class) => {
                    let mut iter = input.chars();
                    if let Some(first) = iter.next() {
                        if char_class.contains(&first) {
                            Some(vec![iter.as_str()])
                        } else { None }
                    } else { None }
                }
                RE::NegClass(char_class) => {
                    let mut iter = input.chars();
                    if let Some(first) = iter.next() {
                        if !char_class.contains(&first) {
                            Some(vec![iter.as_str()])
                        } else { None }
                    } else { None }
                }
                RE::Group(index, inner) => {
                    let mut next_inputs = vec![];
                    if let Some(rests) = match_re(vec![input], inner, additional) {
                        for rest in rests {
                            let matched_len = input.len() - rest.len();
                            if matched_len == 0 { continue; }
                            let captured = &input[..matched_len];
                            additional.group_refs
                                .entry(*index)
                                .and_modify(|group| group.push(captured))
                                .or_insert(vec![captured]);
                            next_inputs.push(rest);
                        }
                    }
                    if next_inputs.is_empty() { None } else { Some(next_inputs) }
                }
                RE::Alt(alts) => {
                    let mut res: Vec<&str> = vec![];
                    for alt in alts {
                        if let Some(alt_next_inputs) = match_re(
                            vec![input],
                            alt,
                            additional,
                        ) {
                            res.extend(alt_next_inputs);
                        }
                    }
                    if res.is_empty() { None } else { Some(res) }
                }
                RE::Literal(c) => {
                    let mut iter = input.chars();
                    if let Some(input_c) = iter.next() {
                        if *c == input_c {
                            Some(vec![iter.as_str()])
                        } else { None }
                    } else { None }
                }
                _ => None
            }
        })
        .flatten()
        .collect::<Vec<&str>>();

    if next_inputs.is_empty() {
        None
    } else {
        dbg!(&next_inputs);
        Some(next_inputs)
    }
}
