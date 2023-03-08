use colored::Colorize;

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




macro_rules! logger {
    ( $log_name:literal, $( $arg:expr )* $(,)* ) => {
        {
            let mut _ss = String::from($log_name);
            $( _ss.push_str(&format!("{}", $arg)); )*
            _ss
        }
    };
}

macro_rules! log {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("❔ [INFO]", $($arg)*));
    };
}

macro_rules! info {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("❔ [INFO]", $($arg)*).blue());
    };
}

macro_rules! debug {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("🐛 [DEBUG]", $($arg)*).magenta());
    };
}

macro_rules! success {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("✅ [SUCCESS]", $($arg)*).green());
    };
}

macro_rules! error {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("❌ [ERROR]", $($arg)*).red());
    };
}

macro_rules! fatal {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("📛 [FATAL]", $($arg)*).red());
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_test() {
        log!("Just logging some stuff...", 123, 5);
        info!("Repairing cache directory...", "/dir/name");
        debug!("Debugging cache...");
        error!("There was a tiny error!");
        fatal!("There was humungous error!!");
        success!("Directory fixed!");
    }
}