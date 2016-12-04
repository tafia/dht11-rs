
#![allow(missing_docs)]

error_chain! {
    types {
        Error, ErrorKind, Result;
    }

//     links {
//         rustup_dist::Error, rustup_dist::ErrorKind, Dist;
//         rustup_utils::Error, rustup_utils::ErrorKind, Utils;
//     }

    foreign_links {
        ::sysfs_gpio::Error, Gpio;
        ::std::time::SystemTimeError, Time;
    }

//     errors {
//         InvalidToolchainName(t: String) {
//             description("invalid toolchain name")
//             display("invalid toolchain name: '{}'", t)
//         }
//     }
}
