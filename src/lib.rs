#[macro_use]
extern crate syslog_ng_common;
#[macro_use]
extern crate log;
extern crate regex;

use std::marker::PhantomData;
use syslog_ng_common::{LogMessage, Parser, ParserBuilder, OptionError, Pipe};
use regex::Regex;

// Example: "seq: 0000000000, thread: 0000, runid: 1456947132, stamp: 2016-03-02T20:32:12 PAD"
pub const LOGGEN_EXPR: &'static str = r"seq: (?P<seq>\d+), thread: (?P<thread>\d+), runid: (?P<runid>\d+), stamp: (?P<stamp>[^ ]+) (?P<padding>.*$)";
pub const REGEX_OPTION: &'static str = "regex";

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct RegexParser {
    pub regex: Regex,
}

pub struct RegexParserBuilder<P: Pipe> {
    regex: Option<Regex>,
    _marker: PhantomData<P>
}

impl<P: Pipe> ParserBuilder<P> for RegexParserBuilder<P> {
    type Parser = RegexParser;
    fn new() -> Self {
        RegexParserBuilder { regex: None, _marker: PhantomData }
    }
    fn option(&mut self, name: String, value: String) {
        if name == REGEX_OPTION {
            debug!("Trying to compile regular expression: '{}'", &value);
            match Regex::new(&value) {
                Ok(regex) => self.regex = Some(regex),
                Err(err) => error!("{}", err)
            }
        }
    }
    fn build(self) -> Result<Self::Parser, OptionError> {
        debug!("Building Regex parser");
        if let Some(regex) = self.regex {
            Ok(RegexParser { regex: regex })
        } else {
            Err(OptionError::missing_required_option(REGEX_OPTION))
        }
    }
}

impl<P: Pipe> Parser<P> for RegexParser {
    fn parse(&mut self, _: &mut P, logmsg: &mut LogMessage, input: &str) -> bool {
        if let Some(captures) = self.regex.captures(input) {
            for (name, value) in captures.iter_named() {
                if let Some(value) = value {
                    logmsg.insert(name, value);
                }
            }
            true
        } else {
            false
        }
    }
}

native_parser_proxy!(RegexParserBuilder<LogParser>);

macro_rules! u8_to_c_str {
    ($value:expr) => {
        $value as *const _ as *const i8,
    }
}

macro_rules! count_exprs {
    () => (0);
    ($head:expr) => (1);
    ($head:expr, $($tail:expr),*) => (1 + count_exprs!($($tail),*));
}

// module_info!(
//     canonical_name => "regex-parser\0",
//     version => "3.8.0alpha0\0",
//     description => "A regex parser for syslog-ng\0",
//     core_revision => "3.8\0",
//     plugins => [
//         parser_plugin!(
//             name => "regex-rs\0",
//             builder => RegexParserBuilder<LogParser>
//         )
//     ]
// );

// https://danielkeep.github.io/tlborm/book/blk-counting.html#recursion
//macro_rules! count_tts {
//    () => {0usize};
//    ($_head:tt $($tail:tt)*) => {1usize + count_tts!($($tail)*)};
//}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}

macro_rules! module_info {
    (
        canonical_name: $canonical_name:expr,
        version: $version:expr,
        description: $description:expr,
        core_revision: $core_revision:expr,
        module_init: $module_init:expr,
        plugins: [ $( { $($plugins:expr)* } ),* $(,)* ] $(,)*
    ) => {
        use syslog_ng_common::sys::module::*;
        use syslog_ng_common::sys::GlobalConfig;
        use std::os::raw::{c_int, c_void};

        #[link(name = "syslog-ng-native-connector")]
        extern "C" {
            static native_parser: CfgParser;
        }

        const PLUGINS_LEN: usize = count_tts!($( ($($plugins)*) )*);

        pub static PLUGINS: [Plugin; PLUGINS_LEN] = [ $( $($plugins)* ),* ];

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        pub static module_info: ModuleInfo = ModuleInfo {
            canonical_name: $canonical_name as *const _ as *const i8,
            version: $version as *const _ as *const i8,
            description: $description as *const _ as *const i8,
            core_revision: $core_revision as *const _ as *const i8,
            plugins: &PLUGINS[0] as *const Plugin,
            plugins_len: PLUGINS_LEN as i32,
            preference: 0,
        };

        #[no_mangle]
        pub fn $module_init(cfg: *mut GlobalConfig, _: *mut CfgArgs) -> c_int
        {
            unsafe { plugin_register(cfg, PLUGINS.as_ptr(), PLUGINS.len() as i32); }
            1
        }
    };
}

macro_rules! parser_plugin {
    (
        name: $name:expr,
        //builder: $builder:ty $(,)*
    ) => {
        Plugin {
            _type: TokenType::ContextParser as i32,
            name: $name as *const _ as *const i8,
            parser: &native_parser,
            setup_context: 0 as *const c_void,
            construct: 0 as *const c_void,
            free_fn: 0 as *const c_void,
        }
    };
}

module_info!(
    canonical_name: b"regex-parser\0",
    version: b"3.8.0alpha0",
    description: b"regex parser for syslog-ng\0",
    core_revision: b"3.8\0",
    module_init: regex_parser_module_init,
    plugins: [
        parser_plugin!(
            name: b"regex-rs\0",
        )
    ]
);

//
//
//
// use syslog_ng_common::sys::module::*;
// use syslog_ng_common::sys::GlobalConfig;
// use std::os::raw::{c_int, c_void};
//
// #[link(name = "syslog-ng-native-connector")]
// extern "C" {
//     static native_parser: CfgParser;
// }
//
// const PLUGINS_LEN: usize = 1;
//
// pub static PLUGINS: [Plugin; PLUGINS_LEN] = [
//     Plugin {
//         _type: TokenType::ContextParser as i32,
//         name: b"regex-rs\0" as *const _ as *const i8,
//         parser: &native_parser,
//         setup_context: 0 as *const c_void,
//         construct: 0 as *const c_void,
//         free_fn: 0 as *const c_void,
//     },
// ];
//
// #[no_mangle]
// #[allow(non_upper_case_globals)]
// pub static module_info: ModuleInfo = ModuleInfo {
//     canonical_name: b"regex-parser\0" as *const _ as *const i8,
//     version: b"3.8.0alpha0\0" as *const _ as *const i8,
//     description: b"description\0" as *const _ as *const i8,
//     core_revision: b"3.8\0" as *const _ as *const i8,
//     plugins: &PLUGINS as *const _ as *const Plugin,
//     plugins_len: PLUGINS_LEN as i32,
//     preference: 0
// };
//
// #[no_mangle]
// pub fn regex_parser_module_init(cfg: *mut GlobalConfig, _: *mut CfgArgs) -> c_int
// {
//     unsafe { plugin_register(cfg, PLUGINS.as_ptr(), PLUGINS.len() as i32); }
//     1
// }
