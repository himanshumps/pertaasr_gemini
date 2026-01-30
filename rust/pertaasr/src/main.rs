mod reqwest_client;
mod utils;
mod constants;

use std::env;
use j4rs::{InvocationArg, JavaOpt, JvmBuilder};
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use humantime::format_duration;
use crate::constants::{JAVA_HOME, RUN_DURATION, VUSER_COUNT};
use crate::utils::get_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up JAVA_HOME if one doesn't exist (for local testing)
    let jdk25 = PathBuf::from(
        "/Users/himanshu/zulu25",
    );
    if jdk25.exists() {
        unsafe {
            env::set_var("JAVA_HOME", jdk25.to_str().unwrap());
        }
    }

    // Read vuser count and run duration from environment variables
    VUSER_COUNT.get_or_init( || {
        get_env("VUSER_COUNT", "20").parse::<usize>().unwrap()
    });
    println!("Total number of users for which test will be running: {}", VUSER_COUNT.get().unwrap());
    RUN_DURATION.get_or_init(|| {
        get_env("RUN_DURATION", "120").parse::<u64>().unwrap()
    });
    println!("The duration for which test will be running: {}", format_duration(Duration::from_secs(RUN_DURATION.get().unwrap().clone())));
    // This is not really needed but added for local testing/verbosity
    JAVA_HOME.get_or_init(|| {
        get_env("JAVA_HOME", "/Users/himanshu/zulu25")
    });
    // Set up JAVA_HOME if one doesn't exist (for local testing)
    let jvm = JvmBuilder::new()
        .java_opt(JavaOpt::new("--enable-native-access=ALL-UNNAMED"))
        .java_opt(JavaOpt::new("--sun-misc-unsafe-memory-access=allow"))
        .build()?;
    {
        let version_prop = InvocationArg::try_from("java.version")?;
        let java_vendor = InvocationArg::try_from("java.vendor")?;
        let version_instance = jvm.invoke_static(
            "java.lang.System", // Class name
            "getProperty",      // Method name
            &[version_prop],    // Arguments
        )?;
        let java_vendor_instance = jvm.invoke_static(
            "java.lang.System", // Class name
            "getProperty",      // Method name
            &[java_vendor],    // Arguments
        )?;
        // 4. Convert the returned Java String instance back to a Rust String
        let version_str: String = jvm.to_rust(version_instance)?;
        let java_vendor_str: String = jvm.to_rust(java_vendor_instance)?;
        println!("Java: {} {}", java_vendor_str, version_str);
    }

    let resp = reqwest_client::build_request_client(VUSER_COUNT.get().unwrap().clone())
        .get("https://httpbin.org/ip")
        .send()
        .await?;
    println!("{:#?}", resp.bytes().await?);
    Ok(())
}
