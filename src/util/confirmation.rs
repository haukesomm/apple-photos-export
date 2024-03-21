use std::io::{stdin, stdout, Write};

use crate::util::confirmation::Answer::{No, Yes};

const CHARACTER_YES: char = 'y';
const CHARACTER_NO: char = 'n';

#[derive(PartialEq)]
pub enum Answer {
    Yes,
    No
}

pub fn confirmation_prompt(prompt: String) -> Answer {
    let mut input = String::new();

    while input.len() != 1 || match input.chars().next() {
        Some(CHARACTER_YES) | Some(CHARACTER_NO) => false,
        _ => true
    } {
        input.clear();

        print!("{} [{}{}] ", prompt, CHARACTER_YES, CHARACTER_NO);
        let _ = stdout().flush();
        stdin().read_line(&mut input).expect("Unable to read user input");

        if Some('\n') == input.chars().next_back() {
            input.pop();
        }
        if Some('\r') == input.chars().next_back() {
            input.pop();
        }
    }

    match input.chars().next_back() {
        Some(CHARACTER_YES) => Yes,
        Some(CHARACTER_NO) => No,
        _ => unreachable!()
    }
}