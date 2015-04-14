// De-duplication macro used in src/app.rs
macro_rules! get_help {
	($opt:ident) => {
		if let Some(h) = $opt.help {
	        format!("{}{}", h,
	            if let Some(ref pv) = $opt.possible_vals {
	                let mut pv_s = pv.iter().fold(String::with_capacity(50), |acc, name| acc + &format!(" {}",name)[..]);
	                pv_s.shrink_to_fit();
	                format!(" [values:{}]", &pv_s[..])
	            }else{"".to_owned()})
	    } else {
	        "    ".to_owned()
	    } 
	};
}

// Thanks to bluss and flan3002 in #rust IRC
//
// Helps with rightward drift when iterating over something and matching each item.
macro_rules! for_match {
	($it:ident, $($p:pat => $($e:expr);+),*) => {
		for i in $it {
			match i {
			$(
			    $p => { $($e)+ }
			)*
			}
		}
	};
}

/// Convenience macro getting a typed value `T` where `T` implements `std::fmt::FrmStr`
/// This macro returns a `Result<T,String>` which allows you as the developer to decide
/// what you'd like to do on a failed parse. There are two types of errors, parse failures
/// and those where the argument wasn't present (such as a non-required argument). 
///
/// You can use it to get a single value, or a `Vec<T>` with the `values_of()`
/// 
/// **NOTE:** Be cautious, as since this a macro invocation it's not exactly like
/// standard syntax.
///
///
/// # Example single value
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # use clap::App;
/// # fn main() {
/// let matches = App::new("myapp")
///               .arg_from_usage("[length] 'Set the length to use as a pos whole num, i.e. 20'")
///				  .get_matches();
/// let len = value_t!(matches.value_of("length"), u32)
/// 				.unwrap_or_else(|e|{
///						println!("{}",e); 
///						std::process::exit(1)
///					});
///
/// println!("{} + 2: {}", len, len + 2);
/// # }
/// ```
///
///
/// # Example multiple values
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # use clap::App;
/// # fn main() {
/// let matches = App::new("myapp")
///               .arg_from_usage("[seq]... 'A sequence of pos whole nums, i.e. 20 45'")
///				  .get_matches();
/// for v in value_t!(matches.values_of("seq"), u32)
///				.unwrap_or_else(|e|{
///					println!("{}",e); 
///					std::process::exit(1)
///				}) {
/// 	println!("{} + 2: {}", v, v + 2);
///	}
/// # }
/// ```
#[macro_export]
macro_rules! value_t {
	($m:ident.value_of($v:expr), $t:ty) => {
		match $m.value_of($v) {
			Some(v) => {
				match v.parse::<$t>() {
					Ok(val) => Ok(val),
					Err(_)  => Err(format!("{} isn't a valid {}",v,stringify!($t))),
				}
			},
			None => Err(format!("Argument \"{}\" not found", $v))
		}
	};
	($m:ident.values_of($v:expr), $t:ty) => {
		match $m.values_of($v) {
			Some(ref v) => {
				let mut tmp = Vec::with_capacity(v.len());
				let mut err = None;
				for pv in v {
					match pv.parse::<$t>() {
						Ok(rv) => tmp.push(rv),
						Err(_) => {
							err = Some(format!("{} isn't a valid {}",pv,stringify!($t)));
							break
						}
					}
				}
				match err {
					Some(e) => Err(e),
					None => Ok(tmp)
				}
			},
			None => Err(format!("Argument \"{}\" not found", $v))
		}
	};
}

/// Convenience macro getting a typed value `T` where `T` implements `std::fmt::FrmStr`
/// This macro returns a `T` or `Vec<T>` or exits with a usage string upon failure. This
/// removes some of the boiler plate to handle failures from value_t! above. 
///
/// You can use it to get a single value `T`, or a `Vec<T>` with the `values_of()`
/// 
/// **NOTE:** This should only be used on required arguments, as it can be confusing to the user
/// why they are getting error messages when it appears they're entering all required argumetns.
///
/// **NOTE:** Be cautious, as since this a macro invocation it's not exactly like
/// standard syntax.
///
///
/// # Example single value
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # use clap::App;
/// # fn main() {
/// let matches = App::new("myapp")
///               .arg_from_usage("[length] 'Set the length to use as a pos whole num, i.e. 20'")
///				  .get_matches();
/// let len = value_t_or_exit!(matches.value_of("length"), u32);
///
/// println!("{} + 2: {}", len, len + 2);
/// # }
/// ```
///
///
/// # Example multiple values
///
/// ```no_run
/// # #[macro_use]
/// # extern crate clap;
/// # use clap::App;
/// # fn main() {
/// let matches = App::new("myapp")
///                   .arg_from_usage("[seq]... 'A sequence of pos whole nums, i.e. 20 45'")
///					  .get_matches();
/// for v in value_t_or_exit!(matches.values_of("seq"), u32) {
/// 	println!("{} + 2: {}", v, v + 2);
///	}
/// # }
/// ```
#[macro_export]
macro_rules! value_t_or_exit {
	($m:ident.value_of($v:expr), $t:ty) => {
		match $m.value_of($v) {
			Some(v) => {
				match v.parse::<$t>() {
					Ok(val) => val,
					Err(_)  => {
						println!("{} isn't a valid {}\n{}\nPlease re-run with --help for more information",
							v,
							stringify!($t), 
							$m.usage());
						::std::process::exit(1);
					}
				}
			},
			None => {
				println!("Argument \"{}\" not found or is not valid\n{}\nPlease re-run with --help for more information",
					$v, 
					$m.usage());
				::std::process::exit(1);
			}
		}
	};
	($m:ident.values_of($v:expr), $t:ty) => {
		match $m.values_of($v) {
			Some(ref v) => {
				let mut tmp = Vec::with_capacity(v.len());
				for pv in v {
					match pv.parse::<$t>() {
						Ok(rv) => tmp.push(rv),
						Err(_)  => {
							println!("{} isn't a valid {}\n{}\nPlease re-run with --help for more information",
								pv,
								stringify!($t), 
								$m.usage()); 
							::std::process::exit(1);
						}
					}
				}
				tmp
			},
			None => {
				println!("Argument \"{}\" not found or is not valid\n{}\nPlease re-run with --help for more information",
					$v, 
					$m.usage());
				::std::process::exit(1);
			}
		}
	};
}