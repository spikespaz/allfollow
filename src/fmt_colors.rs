//! If you borrow this, please give me credit.

#[macro_export]
macro_rules! log {
    ( $($args:tt)* ) => {
        print!("{}", $crate::format_args_colored!( $($args)* ))
    };
}

#[macro_export]
macro_rules! logln {
    ( $($args:tt)* ) => {
        println!("{}", $crate::format_args_colored!( $($args)* ))
    };
}

#[macro_export]
macro_rules! elog {
    ( $($args:tt)* ) => {
        eprint!("{}", $crate::format_args_colored!( $($args)* ))
    };
}

#[macro_export]
macro_rules! elogln {
    ( $($args:tt)* ) => {
        eprintln!("{}", $crate::format_args_colored!( $($args)* ))
    };
}

/// Colored format literals with custom syntax.
#[macro_export]
macro_rules! format_args_colored {
    // By default, all literals are treated as format strings.
    ( $(: $style:ident)* $format:literal $($tail:tt)* ) => {
        format_args!( "{}{}", format_args!( $format ) $(.$style())* , $crate::format_args_colored!( $($tail)* ) )
    };

    // To display a literal value, escape it with a reference/ampersand.
    ( $(: $style:ident)* & $literal:literal $($tail:tt)* ) => {
        format_args!( "{}{}", $literal $(.$style())* , $crate::format_args_colored!( $($tail)* ) )
    };

    // Bare identifiers are not allowed, but references are.
    ( $(: $style:ident)* & $ident:ident $($tail:tt)* ) => {
        format_args!( "{}{}", ( & $ident ) $(.$style())* , $crate::format_args_colored!( $($tail)* ) )
    };

    // Expressions must be placed in parentheses.
    ( $(: $style:ident)* ( $expr:expr ) $($tail:tt)* ) => {
        format_args!( "{}{}", ( $expr ) $(.$style())* , $crate::format_args_colored!( $($tail)* ) )
    };

    // Inline expression blocks are allowed.
    ( $(: $style:ident)* { $($expr:tt)+ } $($tail:tt)* ) => {
        format_args!( "{}{}", { $($expr)+ } $(.$style())* , $crate::format_args_colored!( $($tail)* ) )
    };

    // Bare parentheses with multiple items will recurse colored formatting.
    ( $(: $style:ident)* ( $($recurse:tt)+ ) $($tail:tt)* ) => {
        format_args!( "{}{}", format_args_colored!( $($recurse)+ ) $(.$style())* , $crate::format_args_colored!( $($tail)* ) )
    };

    // Parentheses prefixed with a period are treated as normal format arguments.
    ( $(: $style:ident)* . ( $($format_args:tt)+ ) $($tail:tt)* ) => {
        format_args!( "{}{}", format_args!( $($format_args)+ ) $(.$style())* , $crate::format_args_colored!( $($tail)* ) )
    };

    // Comma has not been matched by previous recursion, prepend a space.
    ( , $($tail:tt)* ) => {
        format_args!( " {}", $crate::format_args_colored!( $($tail)* ))
    };

    // Semicolon has not been matched by previous recursion, prepend a newline.
    ( ; $($tail:tt)* ) => {
        format_args!( "\n{}", $crate::format_args_colored!( $($tail)* ))
    };

    // Terminated.
    () => { "" };
}

#[cfg(test)]
mod tests {
    use crate::elogln;
    use owo_colors::OwoColorize;

    struct NoCopy(Vec<String>);

    impl std::fmt::Display for NoCopy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0.join("+"))
        }
    }

    #[test]
    fn compile() {
        elogln!(:bold :red "text");
        elogln!(:bold :red &10);
        elogln!(:bold :red (1 + 1));
        elogln!(:bold :red (1 + 1), :red (5 + 5));
        let large_number = usize::MAX;
        elogln!(:bold :red (large_number));
        elogln!(:bold :red (large_number), :green (large_number));
        elogln!(:bold :red &large_number, :green &large_number);
        elogln!(:red "separated", :green "with", :blue "spaces");
        elogln!(:dimmed :black :on_bright_white "no" "space");
        elogln!(:black :on_white "no" :white :on_black "space");
        elogln!(
            :bold "first line";
            :bold :blue "second line"
        );
        elogln!(:yellow "formatted {large_number}", :red (usize::MAX));
        let no_copy = NoCopy(vec!["no".to_owned(), "copy".to_owned()]);
        elogln!(:yellow (no_copy), :red (usize::MAX));
        elogln!(:on_bright_white (
            :black "black"
            :yellow "yellow"
        ));
        elogln!("bare ident", { 5 + 5 });
        elogln!("bare ident", {
            let a = 5;
            let b = 5;
            a + b
        });
        elogln!(:green .("This is {} {} {}", "normal", "formatted", "text"))
    }
}
