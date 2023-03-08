macro_rules! cmd {
    ( $program:expr ) => {
        std::process::Command::new($program)
            .output()
            .expect(&format!(
                "CMD_EXEC_FAIL: {}",
                $program
            ))
    };
    ( $program:expr, [$($arg:expr $(,)*)*] ) => {
        std::process::Command::new($program)
            $( .arg($arg) )*
            .output()
            .expect(&format!(
                "CMD_EXEC_FAIL: '{} {}'",
                vec![ $($arg,)* ].iter().fold(String::new(), |acc, s| acc + s + ""),
                $program
            ))
    };
    ( $program:expr, [$($arg:expr $(,)*)*], $msg:literal ) => {
        std::process::Command::new($program)
            $( .arg($arg) )*
            .output()
            .expect(&format!(
                "CMD_EXEC_FAIL: '{} {}' {}",
                $program,
                vec![ $($arg,)* ].iter().fold(String::new(), |acc, s| acc + s + ""),
                $msg
            ))
    };
}