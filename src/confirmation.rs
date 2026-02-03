//! Some basic utilities to prompt the user for confirmation.
//!
//! Everything in here is quite a mess and probably the ugliest part of this application ;-)

use std::io::{stdin, stdout, Read, Write};

const CHARACTER_YES: char = 'y';
const CHARACTER_NO: char = 'n';

pub enum Answer {
    Yes,
    No,
}

pub fn wait_for_enter_key_press() {
    print!("Press enter to continue...");
    let _ = stdout().flush();
    let _ = stdin().read(&mut []);
}

pub fn confirmation_prompt(prompt: &str) -> Answer {
    let mut input = String::new();

    while input.len() != 1
        || match input.chars().next() {
            Some(CHARACTER_YES) | Some(CHARACTER_NO) => false,
            _ => true,
        }
    {
        input.clear();

        print!("{} [{}{}] ", prompt, CHARACTER_YES, CHARACTER_NO);
        let _ = stdout().flush();
        stdin()
            .read_line(&mut input)
            .expect("Unable to read user input");

        if Some('\n') == input.chars().next_back() {
            input.pop();
        }
        if Some('\r') == input.chars().next_back() {
            input.pop();
        }
    }

    match input.chars().next_back() {
        Some(CHARACTER_YES) => Answer::Yes,
        Some(CHARACTER_NO) => Answer::No,
        _ => unreachable!(),
    }
}
