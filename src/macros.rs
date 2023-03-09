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
            $( _ss.push_str(&format!("{} ", {$arg})); )*
            _ss
        }
    };
}

macro_rules! log {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("   ", $({$arg})*));
    };
}

macro_rules! info {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("❔ ", $({$arg})*).blue());
    };
}

macro_rules! debug {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("🐛 ", $({$arg})*).magenta());
    };
}

macro_rules! success {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}{}", logger!("✅ ", $({$arg})*).green(), "\n");
    };
}

macro_rules! warn {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("⚠️  ", $({$arg})*).yellow());
    };
}

macro_rules! error {
    ( $( $arg:expr $(,)*)* ) => {
        println!("{}", logger!("❌ ", $({$arg})*).red());
    };
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_test() {
        log!("Just logging some stuff...", 123, 5);
        info!("Repairing cache directory...", "/dir/name");
        debug!("Debugging cache...", "Bug hiding here!", 345, "🐛");
        warn!("Careful before proceeding!", "Yellow Stains");
        error!("There was a tiny error!", "Code: ", 0x3276);
        success!("Directory fixed!");
    }
}