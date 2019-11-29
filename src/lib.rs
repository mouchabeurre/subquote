pub mod io {
  use std::env;
  use std::fmt::{self, Debug, Display, Formatter};
  use std::path;
  use std::fs;
  use getopts::Options;

  pub enum ParseOutcome {
    Error(String),
    Help
  }

  enum ArgProvided<T> {
    Yes(T),
    No(T)
  }

  impl<T> Display for ArgProvided<T>
    where T: Display {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      match self {
        ArgProvided::Yes(t) => write!(f, "{}", t),
        ArgProvided::No(t) => write!(f, "{}", t)
      }
    }
  }

  impl<T> ArgProvided<T> {
    fn get_value(&self) -> &T {
      match self {
        ArgProvided::Yes(t) => t,
        ArgProvided::No(t) => t
      }
    }
  }

  pub enum Unit {
    Grapheme,
    Word
  }

  impl Clone for Unit {
    fn clone(&self) -> Self {
      match self {
        Unit::Word => Unit::Word,
        Unit::Grapheme => Unit::Grapheme
      }
    }
  }

  impl Display for Unit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      match self {
        Unit::Word => write!(f, "word"),
        Unit::Grapheme => write!(f, "grapheme"),
      }
    }
  }

  pub struct SafeArguments {
    pub subtitle: String,
    pub quote_length: i32,
    pub verbosity: bool,
    pub unit: Unit,
    pub cache_directory: String,
  }

  impl SafeArguments {
    fn new(
      subtitle: String,
      quote_length: i32,
      verbosity: bool,
      cache_directory: String,
      unit: Unit
    ) -> Self {
      Self {
        subtitle,
        quote_length,
        verbosity,
        cache_directory,
        unit
      }
    }
  }

  pub struct UnsafeArguments {
    subtitle: String,
    quote_length: ArgProvided<i32>,
    verbosity: ArgProvided<bool>,
    unit: ArgProvided<Unit>,
    cache_directory: ArgProvided<String>,
  }

  impl Debug for UnsafeArguments {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      write!(
        f,
        "UnsafeArguments {{ subtitle: {}, quote_length: {}, verbosity: {}, cache_directory: {}, unit: {} }}",
        self.subtitle,
        self.quote_length,
        self.verbosity,
        self.cache_directory,
        self.unit
      )
    }
  }

  impl UnsafeArguments {
    fn new(
      subtitle: String,
      quote_length: ArgProvided<i32>,
      verbosity: ArgProvided<bool>,
      cache_directory: ArgProvided<String>,
      unit: ArgProvided<Unit>
    ) -> Self {
      Self {
        subtitle,
        quote_length,
        verbosity,
        cache_directory,
        unit
      }
    }
    fn get_default_quote_length(unit: Option<Unit>) -> i32 {
      let l_word: i32 = 5;
      let l_grapheme: i32 = 25;
      match unit {
        Some(unit) => {
          match unit {
            Unit::Word => l_word,
            Unit::Grapheme => l_grapheme
          }
        },
        None => {
          match Self::get_default_unit() {
            Unit::Word => l_word,
            Unit::Grapheme => l_grapheme
          }
        }
      }
    }
    fn get_default_verbosity() -> bool { false }
    fn get_default_unit() -> Unit { Unit::Word }
    fn get_default_cache_directory() -> Option<String> {
      match env::var_os("XDG_CACHE_HOME") {
        Some(p_os_str) => match p_os_str.into_string() {
          Ok(p_str) => return Some(format!("{}/subquote", p_str)),
          Err(_) => return None
        },
        None => return None
      };
    }
    pub fn validate(&self) -> Result<SafeArguments, String> {
      let mut errors: Vec<String> = Vec::new();
      if *self.quote_length.get_value() < 1 {
        errors.push(format!(
          "quote length must be greater or equal to 1 (got \"{}\")", &self.quote_length)
        )
      }
      if !path::Path::new(&self.cache_directory.get_value()).is_dir() {
        match &self.cache_directory {
          ArgProvided::Yes(_) => {
            errors.push(format!(
              "couldn't read specified cache directory (got \"{}\")", &self.cache_directory)
            )
          },
          ArgProvided::No(dir) => {
            match fs::create_dir(path::Path::new(&dir)) {
              Ok(_) => {
                if *self.verbosity.get_value() {
                  println!("Created default cache directory at {}", &dir);
                }
              },
              Err(_) => errors.push(
                format!("couldn't create cache directory (got \"{}\")", &self.cache_directory)
              )
            }
          }
        }
      } 
      if !path::Path::new(&self.subtitle).is_file() {
        errors.push(format!("specified subtitle is not a file (got \"{}\")", &self.subtitle))
      }
      if errors.len() > 0 {
        return Err(errors.join("; "));
      }
      Ok(SafeArguments::new(
        self.subtitle.clone(),
        self.quote_length.get_value().clone(),
        self.verbosity.get_value().clone(),
        self.cache_directory.get_value().clone(),
        (*self.unit.get_value()).clone()
      ))
    }
  }

  fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE.srt [options]", program);
    println!("{}", opts.usage(&brief));
  }

  pub fn parse_args(args: &[String]) -> Result<UnsafeArguments, ParseOutcome> {
    let program = &args[0];
    let def_quote_length = UnsafeArguments::get_default_quote_length(None);
    let def_verbosity = UnsafeArguments::get_default_verbosity();
    let def_unit = UnsafeArguments::get_default_unit();
    let def_cache_directory = UnsafeArguments::get_default_cache_directory();

    let desc_quote_length = format!("Maximum quote length (default: {})", def_quote_length);
    let desc_verbosity = format!("Be verbose (default: {})", def_verbosity);
    let desc_unit = format!("Unit used to build the quote: \"word\" or \"char\" (default: {})", def_unit);
    let desc_help = String::from("Print this help menu");
    let desc_cache_base = String::from("Specify where to save processed subtitles");
    let desc_cache_directory = match def_cache_directory.clone() {
      Some(p_str) => {
        format!("{} (default {})", desc_cache_base, p_str)
      },
      None => desc_cache_base
    };

    let mut opts = Options::new();
    let opt_l = ("l", "length", &desc_quote_length);
    let opt_u = ("u", "unit", &desc_unit);
    let opt_c = ("", "cache", &desc_cache_directory);
    let opt_v = ("v", "", &desc_verbosity);
    let opt_h = ("h", "help", &desc_help);
    opts.optopt(opt_l.0, opt_l.1, opt_l.2, "");
    opts.optopt(opt_u.0, opt_u.1, opt_u.2, "");
    opts.optopt(opt_c.0, opt_c.1, opt_c.2, "");
    opts.optflag(opt_v.0, opt_v.1, opt_v.2);
    opts.optflag(opt_h.0, opt_h.1, opt_h.2);
    let matches = match opts.parse(&args[1..]) {
        Ok(opt) => opt,
        Err(_) => {
          return Err(ParseOutcome::Error(
          String::from("found incomplete or unsupported arguments")))
        }
    };
    if matches.opt_present(opt_h.0) {
        print_usage(&program, opts);
        return Err(ParseOutcome::Help);
    }

    let verbosity = match matches.opt_present(opt_v.0) {
      true => ArgProvided::Yes(true),
      false => ArgProvided::No(def_verbosity)
    };
    let unit = match matches.opt_str(opt_u.0) {
      Some(unit) => {
        match unit.as_str() {
          "word" => ArgProvided::Yes(Unit::Word),
          "char" => ArgProvided::Yes(Unit::Grapheme),
          _ => return Err(ParseOutcome::Error(
            format!("couldn't parse specified {}", &opt_u.1))
          )
        }
      },
      None => ArgProvided::No(def_unit)
    };
    let quote_length = match matches.opt_str(opt_l.0) {
      Some(len) => match len.parse::<i32>() {
        Ok(len) => ArgProvided::Yes(len),
        Err(_) => return Err(ParseOutcome::Error(
          format!("couldn't parse specified {}", &opt_l.1))
        )
      },
      None => {
        match unit.get_value() {
          Unit::Word => ArgProvided::No(UnsafeArguments::get_default_quote_length(Some(Unit::Word))),
          Unit::Grapheme => ArgProvided::No(UnsafeArguments::get_default_quote_length(Some(Unit::Grapheme)))
        }
      }
    };
    let cache_directory = match matches.opt_str(opt_c.1) {
      Some(dir) => ArgProvided::Yes(dir),
      None => match def_cache_directory {
        Some(dir) => ArgProvided::No(dir),
        None => return Err(ParseOutcome::Error(
          format!(
            "couldn't determine user's default cache directory (provide it with --{} /path/to/cache)",
            opt_c.1
          )
        ))
      }
    };
    let subtitle = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
      return Err(ParseOutcome::Error(
        String::from("subtitle file is requiered"))
      )
    };

    Ok(UnsafeArguments::new(subtitle, quote_length, verbosity, cache_directory, unit))
  }
}

pub mod builder {
  use std::fs;
  use std::path;
  use std::collections::HashMap;
  use std::io::BufReader;
  use regex::Regex;
  use serde::{Serialize, Deserialize};
  use serde_json;
  use rand::Rng;
  use super::io::{SafeArguments, Unit};
  use super::builder::Quote::{Node, Nil};

  #[derive(Serialize, Deserialize)]
  struct Entries {
    entries: Vec<Entry>
  }

  impl Entries {
    fn new() -> Self {
      Self {
        entries: Vec::new()
      }
    }
    fn add_entry(&mut self, entry: Entry) {
      self.entries.push(entry)
    }
  }

  #[derive(Serialize, Deserialize)]
  struct Entry {
    key: String,
    pairs: Vec<String>
  }

  impl Entry {
    fn new(key: String, pairs: Vec<String>) -> Self {
      Self { key, pairs }
    }
  }

  enum Quote {
    Node(String, Box<Quote>),
    Nil
  }

  impl Quote {
    fn build(&self) -> Option<Vec<String>> {
      match self {
        Node(unit, next) => {
          match next.build() {
            Some(mut remaining) => {
              let mut current = vec![unit.clone()];
              current.append(remaining.as_mut());
              Some(current)
            },
            None => {
              Some(vec![unit.clone()])
            }
          }
        },
        Nil => None
      }
    }
  }

  pub fn get_quote(args: SafeArguments) -> Result<String, String> {
    let mut cached_dict = path::PathBuf::from(&args.cache_directory);
    let mut split_subtitle_path: Vec<&str> = args.subtitle.split("/").collect();
    let subtitle = match split_subtitle_path.pop() {
      Some(filename) => filename,
      None => return Err(String::from("couldn't determine subtitle filename"))
    };
    cached_dict.push(subtitle);
    let _ = match args.unit {
      Unit::Word => cached_dict.set_extension("word"),
      Unit::Grapheme => cached_dict.set_extension("char"),
    };
    if cached_dict.is_file() {
      let dict = match load_dict(cached_dict) {
        Ok(de_dict) => de_dict,
        Err(err) => return Err(err)
      };
      match generate_quote(&dict, args.quote_length) {
        Ok(quote) => Ok(quote),
        Err(err) => Err(err)
      }
    } else {
      let mut dict: HashMap<String, Vec<String>> = HashMap::new();
      match fs::read_to_string(&args.subtitle) {
        Err(_) => return Err(String::from("couldn't open subtitle file")),
        Ok(subtitle) => {
          let subrip_reg = Regex::new(r"(^\d{2}:\d{2}:\d{2},\d{3}\s-->\s\d{2}:\d{2}:\d{2},\d{3}$)|(^\d+$)|(^()$)")
            .unwrap();
          let quotes_reg = Regex::new(r#""\s?|<.*>\s?|,|-"#)
            .unwrap();
          for line in subtitle.lines() {
            if !subrip_reg.is_match(line) {
              let replaced = quotes_reg.replace_all(line, "");
              let units = match args.unit {
                Unit::Word => replaced.split_whitespace(),
                Unit::Grapheme => replaced.split_whitespace()
              };
              let mut iter = units.peekable();
              loop {
                let next = match iter.next() {
                  Some(next) => String::from(next),
                  None => break
                };
                let peeked = match iter.peek() {
                  Some(next) => String::from(*next),
                  None => break
                };
                match dict.get_mut(&next) {
                  Some(entry) => {
                    entry.push(peeked)
                  },
                  None => {
                    if !next.ends_with(".") {
                      match dict.insert(next, vec![peeked]) {
                        _ => ()
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
      match save_dict(dict.clone(), &cached_dict) {
        Ok(_) => {
          match generate_quote(&dict, args.quote_length) {
            Ok(quote) => Ok(quote),
            Err(err) => Err(err)
          }
        },
        Err(err) => Err(err)
      }
    }
  }

  fn load_dict(cached_dict: path::PathBuf) -> Result<HashMap<String, Vec<String>>, String> {
    let ser_dict = match fs::File::open(cached_dict) {
      Ok(file) => file,
      Err(_) => return Err(String::from("couldn't open cached file"))
    };
    let reader = BufReader::new(ser_dict);
    let de_dict: Entries = match serde_json::from_reader(reader) {
      Ok(dict) => dict,
      Err(_) => return Err(String::from("couldn't deserialize cached file"))
    };
    let mut dict: HashMap<String, Vec<String>> = HashMap::new();
    for entry in de_dict.entries.iter() {
      match dict.insert(entry.key.clone(), entry.pairs.clone()) {
        _ => ()
      }
    }
    Ok(dict)
  }

  fn save_dict(mut dict: HashMap<String, Vec<String>>, file_path: &path::PathBuf) -> Result<(), String> {
    let mut entries = Entries::new();
    dict.drain().for_each(|(key, d_entry)| {
      entries.add_entry(Entry::new(key, d_entry))
    });
    let output = match fs::File::create(file_path) {
      Ok(file) => file,
      Err(_) => return Err(String::from("couldn't create cache file"))
    };
    match serde_json::to_writer(output, &entries) {
      Ok(_) => Ok(()),
      Err(_) => return Err(String::from("couldn't write to cache file"))
    }
  }

  fn generate_quote(dict: &HashMap<String, Vec<String>>, quote_length: i32) -> Result<String, String> {
    let mut starts = dict.keys().filter(|key| {
      let v: Vec<char> = key.chars().collect();
        if v[0].is_uppercase() {
          return true;
        } else {
          return false;
        }
    });
    let random = rand::thread_rng().gen_range(0, starts.clone().count());
    let first = match starts.nth(random) {
      Some(entry) => entry.clone(),
      None => return Err(String::from("couldn't determine the quote starting point"))
    };
    let branch = build_branch(&dict, first, quote_length);
    let mut quote = match branch.build() {
      Some(vec_quote) => vec_quote.join(" "),
      None => return Err(String::from("couldn't build a random quote"))
    };
    let ends_with_reg = Regex::new(r".+[\.,!,\?]$").unwrap();
    if !ends_with_reg.is_match(&quote) {
      quote.push('.');
    }
    Ok(quote)
  }

  fn build_branch(dict: &HashMap<String, Vec<String>>, unit: String, length: i32) -> Quote {
    if length == 0 {
      return Node(unit, Box::new(Nil));
    } else {
      match dict.get(&unit) {
        Some(entry) => {
          let random = rand::thread_rng().gen_range(0, entry.iter().count());
          match entry.iter().nth(random) {
            Some(next) => {
              return Node(unit, Box::new(build_branch(dict, next.clone(), length - 1)))
            },
            None => return Node(unit, Box::new(Nil))
          };
        },
        None => return Node(unit, Box::new(Nil))
      }
    }
  }
}
