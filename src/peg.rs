use self::grammar::job_list;

#[derive(Debug, PartialEq)]
pub struct Job {
    pub command: String,
    pub args: Vec<String>,
}

impl Job {
    fn new(command: String, args: Vec<String>) -> Job {
        Job {
            command: command,
            args: args,
        }
    }
}

pub fn parse(code: &str) -> Vec<Job> {
    job_list(code).unwrap()  // TODO don't unwrap, handle parse error
}

peg! grammar(r#"
use super::Job;


#[pub]
job_list -> Vec<Job>
    = (unused* newline)* jobs:job ++ (job_ending+ unused*) (newline unused*)* { jobs }
    / (unused*) ** newline { vec![] }

job -> Job
    = whitespace? res:_job whitespace? comment? { res }
    
_job -> Job
    = args:word ++ whitespace { let mut args = args.clone(); Job::new(args.remove(0), args) }

word -> String
    = double_quoted_word
    / single_quoted_word
    / [^ \t\r\n#;]+ { match_str.to_string() }

double_quoted_word -> String
    = ["] word:_double_quoted_word ["] { word }

_double_quoted_word -> String
    = [^"]+ { match_str.to_string() }

single_quoted_word -> String
    = ['] word:_single_quoted_word ['] { word }

_single_quoted_word -> String
    = [^']+ { match_str.to_string() }

unused -> ()
    = whitespace comment? { () }
    / comment { () }

comment -> ()
    = [#] [^\r\n]*

whitespace -> ()
    = [ \t]+

job_ending -> ()
    = [;]
    / newline
    / newline

newline -> ()
    = [\r\n]
"#);


#[cfg(test)]
mod tests {
    use super::*;
    use super::grammar::*;

    #[test]
    fn single_job_no_args() {
        let jobs = job_list("cat").unwrap();
        assert_eq!(1, jobs.len());
        assert_eq!("cat", jobs[0].command);
        assert_eq!(0, jobs[0].args.len());
    }

    #[test]
    fn single_job_with_args() {
        let jobs = job_list("ls -al dir").unwrap();
        assert_eq!(1, jobs.len());
        assert_eq!("ls", jobs[0].command);
        assert_eq!("-al", jobs[0].args[0]);
        assert_eq!("dir", jobs[0].args[1]);
    }

    #[test]
    fn multiple_jobs_with_args() {
        let jobs = job_list("ls -al;cat tmp.txt").unwrap();
        assert_eq!(2, jobs.len());
        assert_eq!("ls", jobs[0].command);
        assert_eq!("-al", jobs[0].args[0]);
        assert_eq!("cat", jobs[1].command);
        assert_eq!("tmp.txt", jobs[1].args[0]);
    }

    #[test]
    fn parse_empty_string() {
        let jobs = job_list("").unwrap();
        assert_eq!(0, jobs.len());
    }

    #[test]
    fn multiple_white_space_between_words() {
        let jobs = job_list("ls \t -al\t\tdir").unwrap();
        assert_eq!(1, jobs.len());
        assert_eq!("ls", jobs[0].command);
        assert_eq!("-al", jobs[0].args[0]);
        assert_eq!("dir", jobs[0].args[1]);
    }

    #[test]
    fn trailing_whitespace() {
        let jobs = job_list("ls -al\t ").unwrap();
        assert_eq!(1, jobs.len());
        assert_eq!("ls", jobs[0].command);
        assert_eq!("-al", jobs[0].args[0]);
    }

    #[test]
    fn double_quoting() {
        let jobs = job_list("echo \"Hello World\"").unwrap();
        assert_eq!(1, jobs[0].args.len());
        assert_eq!("Hello World", jobs[0].args[0]);
    }

    #[test]
    fn all_whitespace() {
        let jobs = job_list("  \t ").unwrap();
        assert_eq!(0, jobs.len());
    }

    #[test]
    fn lone_comment() {
        let jobs = job_list("# ; \t as!!+dfa").unwrap();
        assert_eq!(0, jobs.len());
    }

    #[test]
    fn command_followed_by_comment() {
        let jobs = job_list("cat # ; \t as!!+dfa").unwrap();
        assert_eq!(1, jobs.len());
        assert_eq!(0, jobs[0].args.len());
    }

    //#[test]
    //fn comments_in_multiline_script() {
    //    let jobs = job_list("echo\n# a comment;\necho#asfasdf").unwrap();
    //    assert_eq!(2, jobs.len());
    //}

    #[test]
    fn multiple_newlines() {
        let jobs = job_list("echo\n\ncat").unwrap();
        assert_eq!(2, jobs.len());
    }

    #[test]
    fn leading_whitespace() {
        let jobs = job_list("    \techo").unwrap();
        assert_eq!(1, jobs.len());
        assert_eq!("echo", jobs[0].command);
    }

    #[test]
    fn indentation_on_multiple_lines() {
        let jobs = job_list("echo\n  cat").unwrap();
        assert_eq!(2, jobs.len());
        assert_eq!("echo", jobs[0].command);
        assert_eq!("cat", jobs[1].command);
    }

    #[test]
    fn single_quoting() {
        let jobs = job_list("echo '#!!;\"\\'").unwrap();
        assert_eq!("#!!;\"\\", jobs[0].args[0]);
    }

    #[test]
    fn mixed_quoted_and_unquoted() {
        let jobs = job_list("echo '#!!;\"\\' and \t some \"more' 'stuff\"").unwrap();
        assert_eq!("#!!;\"\\", jobs[0].args[0]);
        assert_eq!("and", jobs[0].args[1]);
        assert_eq!("some", jobs[0].args[2]);
        assert_eq!("more' 'stuff", jobs[0].args[3]);
    }

    #[test]
    fn several_blank_lines() {
        let jobs = parse("\n\n\n");
        assert_eq!(0, jobs.len());
    }

    #[test]
    fn full_script() {
        let jobs = job_list(
r#"if a == a
  echo true a == a

  if b != b
    echo true b != b
  else
    echo false b != b

    if 3 > 2
      echo true 3 > 2
    else
      echo false 3 > 2
    fi
  fi
else
  echo false a == a
fi
"#).unwrap();  // Make sure it parses
    }

    #[test]
    fn leading_and_trailing_junk() {
        let jobs = job_list(
r#"

# comment
   # comment
  

    if a == a   
  echo true a == a  # Line ending commment

  if b != b
    echo true b != b
  else
    echo false b != b

    if 3 > 2
      echo true 3 > 2
    else
      echo false 3 > 2
    fi
  fi
else
  echo false a == a
      fi     

# comment

"#).unwrap();  // Make sure it parses
    }


}