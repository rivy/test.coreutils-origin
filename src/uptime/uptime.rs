#![crate_name = "uu_uptime"]

/*
 * This file is part of the uutils coreutils package.
 *
 * (c) Jordi Boggiano <j.boggiano@seld.be>
 * (c) Jian Zeng <anonymousknight86@gmail.com>
 *
 * For the full copyright and license information, please view the LICENSE
 * file that was distributed with this source code.
 */

/* last synced with: cat (GNU coreutils) 8.13 */

extern crate getopts;
extern crate time;

#[macro_use]
extern crate uucore;
// import crate time from utmpx
pub use uucore::libc;
use uucore::libc::time_t;

use getopts::Options;

static NAME: &str = "uptime";
static VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(unix)]
use libc::getloadavg;

#[cfg(windows)]
extern "C" {
    fn GetTickCount() -> libc::uint32_t;
}

pub fn uumain(args: Vec<String>) -> i32 {
    let mut opts = Options::new();

    opts.optflag("v", "version", "output version information and exit");
    opts.optflag("h", "help", "display this help and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => crash!(1, "Invalid options\n{}", f),
    };
    if matches.opt_present("version") {
        println!("{} {}", NAME, VERSION);
        return 0;
    }
    if matches.opt_present("help") || !matches.free.is_empty() {
        println!("{} {}", NAME, VERSION);
        println!();
        println!("Usage:");
        println!("  {0} [OPTION]", NAME);
        println!();
        println!(
            "{}",
            opts.usage(
                "Print the current time, the length of time the system has been up,\n\
                 the number of users on the system, and the average number of jobs\n\
                 in the run queue over the last 1, 5 and 15 minutes."
            )
        );
        return 0;
    }

    print_time();
    let (boot_time, user_count) = process_utmpx();
    let uptime = get_uptime(boot_time);
    if uptime < 0 {
        show_error!("could not retrieve system uptime");

        1
    } else {
        let upsecs = uptime / 100;
        print_uptime(upsecs);
        print_nusers(user_count);
        print_loadavg();

        0
    }
}

#[cfg(unix)]
fn print_loadavg() {
    use libc::c_double;

    let mut avg: [c_double; 3] = [0.0; 3];
    let loads: i32 = unsafe { getloadavg(avg.as_mut_ptr(), 3) };

    if loads == -1 {
        println!();
    } else {
        print!("load average: ");
        for n in 0..loads {
            print!(
                "{:.2}{}",
                avg[n as usize],
                if n == loads - 1 { "\n" } else { ", " }
            );
        }
    }
}

#[cfg(windows)]
fn print_loadavg() {
    // XXX: currently this is a noop as Windows does not seem to have anything comparable to
    //      getloadavg()
}

#[cfg(unix)]
fn process_utmpx() -> (Option<time_t>, usize) {
    use uucore::utmpx::*;

    let mut nusers = 0;
    let mut boot_time = None;

    for line in Utmpx::iter_all_records() {
        match line.record_type() {
            USER_PROCESS => nusers += 1,
            BOOT_TIME => {
                let t = line.login_time().to_timespec();
                if t.sec > 0 {
                    boot_time = Some(t.sec as time_t);
                }
            }
            _ => continue,
        }
    }
    (boot_time, nusers)
}

#[cfg(windows)]
fn process_utmpx() -> (Option<time_t>, usize) {
    (None, 0) // TODO: change 0 to number of users
}

fn print_nusers(nusers: usize) {
    match nusers.cmp(&1) {
        std::cmp::Ordering::Equal => print!("1 user,  "),
        std::cmp::Ordering::Greater => print!("{} users,  ", nusers),
        _ => {}
    };
}

fn print_time() {
    let local_time = time::now();

    print!(
        " {:02}:{:02}:{:02} ",
        local_time.tm_hour, local_time.tm_min, local_time.tm_sec
    );
}

#[cfg(unix)]
fn get_uptime(boot_time: Option<time_t>) -> i64 {
    use std::fs::File;
    use std::io::Read;

    let mut proc_uptime = String::new();

    if let Some(n) = File::open("/proc/uptime")
        .ok()
        .and_then(|mut f| f.read_to_string(&mut proc_uptime).ok())
        .and_then(|_| proc_uptime.split_whitespace().next())
        .and_then(|s| s.replace(".", "").parse().ok())
    {
        n
    } else {
        match boot_time {
            Some(t) => {
                let now = time::get_time().sec;
                let boottime = t as i64;
                ((now - boottime) * 100)
            }
            _ => -1,
        }
    }
}

#[cfg(windows)]
fn get_uptime(_boot_time: Option<time_t>) -> i64 {
    unsafe { GetTickCount() as i64 }
}

fn print_uptime(upsecs: i64) {
    let updays = upsecs / 86400;
    let uphours = (upsecs - (updays * 86400)) / 3600;
    let upmins = (upsecs - (updays * 86400) - (uphours * 3600)) / 60;
    match updays.cmp(&1) {
        std::cmp::Ordering::Equal => print!("up {:1} day, {:2}:{:02},  ", updays, uphours, upmins),
        std::cmp::Ordering::Greater => {
            print!("up {:1} days, {:2}:{:02},  ", updays, uphours, upmins)
        }
        _ => print!("up  {:2}:{:02}, ", uphours, upmins),
    };
}
