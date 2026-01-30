mod constants;
mod ffi;
mod reqwest_client;
mod utils;
mod structs;

use crate::constants::{CONNECTION_COUNT, JAVA_HOME, RUN_DURATION};
use crate::utils::{get_env, init_fory};
use anyhow::Result;
use humantime::format_duration;
use j4rs::{ClasspathEntry, InvocationArg, JavaOpt, JvmBuilder};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use crate::structs::ForyRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up JAVA_HOME if one doesn't exist (for local testing)
    let jdk25 = PathBuf::from("/Users/himanshu/zulu25");
    if jdk25.exists() {
        unsafe {
            env::set_var("JAVA_HOME", jdk25.to_str().unwrap());
        }
    }

    // Read vuser count and run duration from environment variables
    CONNECTION_COUNT.get_or_init(|| get_env("CONNECTION_COUNT", "20").parse::<usize>().unwrap());
    println!(
        "Total number of users for which test will be running: {}",
        CONNECTION_COUNT.get().unwrap()
    );
    RUN_DURATION.get_or_init(|| get_env("RUN_DURATION", "120").parse::<u64>().unwrap());
    println!(
        "The duration for which the test will be running: {}",
        format_duration(Duration::from_secs(RUN_DURATION.get().unwrap().clone()))
    );
    // This is not really needed, but added for local testing/verbosity
    JAVA_HOME.get_or_init(|| get_env("JAVA_HOME", "/Users/himanshu/zulu25"));
    // Set up JAVA_HOME if one doesn't exist (for local testing)
    let jvm = JvmBuilder::new()
        .java_opt(JavaOpt::new("--enable-native-access=ALL-UNNAMED"))
        .java_opt(JavaOpt::new("--sun-misc-unsafe-memory-access=allow"))
        .classpath_entry(ClasspathEntry::new("/Users/himanshu/pertassr_ffi.jar"))
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
            &[java_vendor],     // Arguments
        )?;
        // 4. Convert the returned Java String instance back to a Rust String
        let version_str: String = jvm.to_rust(version_instance)?;
        let java_vendor_str: String = jvm.to_rust(java_vendor_instance)?;
        println!("Java: {} {}", java_vendor_str, version_str);
    }
    // Get the method handle from java using JNI. Once we get the address, we will use FFI to call them
    let get_handle = |class: &str, method: &str| -> Result<usize> {
        let addr: i64 = jvm.to_rust(jvm.invoke_static(class, method, InvocationArg::empty())?)?;
        Ok(addr as usize)
    };
    let init_connection_fn =
        ffi::sync_fn_ptr_from_add_void(get_handle(
            "com.google.gemini.ffi.UpCallMethodStub",
            "getMethodHandleForInitConnection",
        )?).unwrap();
    let fory_supplier_fn =
        ffi::sync_fn_ptr_from_add_void(get_handle(
            "com.google.gemini.ffi.UpCallMethodStub",
            "getMethodHandleForForyRequestSupplier",
        )?).unwrap();
    let memory_segment_address_fn =
        ffi::sync_fn_ptr_from_add_u64(get_handle(
            "com.google.gemini.ffi.UpCallMethodStub",
            "getMethodHandleForMemorySegmentAddress",
        )?).unwrap();

    let mut handles = Vec::new();
    let count = *CONNECTION_COUNT.get().unwrap();
    for i in 0..count {
        init_connection_fn(i as i32);
        let fory = init_fory()?;
        let memory_segment_ptr = memory_segment_address_fn(i as i32) as usize;
        handles.push(tokio::spawn(async move {
            fory_supplier_fn(i as i32); // This will update the memory with fory struct
            let shared_data = unsafe {
                let ptr = memory_segment_ptr as *const u8;
                let length = std::ptr::read_unaligned(ptr).to_le() as usize;
                println!("length: {}", length);
                std::slice::from_raw_parts(ptr.add(4), length)
            };
            let fory_request = match fory.deserialize::<ForyRequest>(&shared_data) {
                Ok(request) => request,
                Err(e) => {
                    panic!("Deserialization issue: {:}", e);
                }
            };
            println!("{:#?}", fory_request);


        }));
    }
    for handle in handles {
        handle.await?;
    }
    Ok(())
}
