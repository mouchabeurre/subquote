
use std::env;
use std::process;
use subquote::{
  io::{self, ParseOutcome},
  builder
};

fn main() {
  let args: Vec<String> = env::args().collect();
  let unsafe_arguments = io::parse_args(&args).unwrap_or_else(|outcome| {
    match outcome {
      ParseOutcome::Error(err) => {
        println!("Error during arguments parsing: {}.", err);
        process::exit(1);
      },
      ParseOutcome::Help => process::exit(1)
    }
  });
  //println!("Parsed args: {:?}", parsed_args);
  let safe_arguments = unsafe_arguments.validate().unwrap_or_else(|err| {
    println!("Error during input validation: [{}].", err);
    process::exit(1);
  });

  let quote = builder::get_quote(safe_arguments).unwrap_or_else(|err| {
    println!("Error during input validation: [{}].", err);
    process::exit(1);
  });
  println!("{}", quote);
}
