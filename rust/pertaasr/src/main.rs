mod constants;
mod ffi;
mod reqwest_client;
mod structs;
mod utils;

use crate::constants::{CONNECTION_COUNT, JAVA_HOME, RUN_DURATION};
use crate::reqwest_client::build_request_client;
use crate::structs::ForyRequest;
use crate::utils::{get_env, init_fory, reqwest_method};
use anyhow::Result;
use humantime::format_duration;
use j4rs::{ClasspathEntry, InvocationArg, JavaOpt, JvmBuilder};
use std::cell::OnceCell;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::Duration;
use tokio::sync::{Barrier};

#[cfg(not(target_env = "msvc"))] // jemalloc is not supported on MSVC targets
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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
    RUN_DURATION.get_or_init(|| get_env("RUN_DURATION", "10").parse::<u64>().unwrap());
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
        .java_opt(JavaOpt::new("-Xms512m"))
        .java_opt(JavaOpt::new("-Xmx512m"))
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
    let init_connection_fn = ffi::sync_fn_ptr_from_add_void(get_handle(
        "com.google.gemini.ffi.UpCallMethodStub",
        "getMethodHandleForInitConnection",
    )?)
    .unwrap();
    let fory_supplier_fn = ffi::sync_fn_ptr_from_add_void(get_handle(
        "com.google.gemini.ffi.UpCallMethodStub",
        "getMethodHandleForForyRequestSupplier",
    )?)
    .unwrap();
    let memory_segment_address_fn = ffi::sync_fn_ptr_from_add_u64(get_handle(
        "com.google.gemini.ffi.UpCallMethodStub",
        "getMethodHandleForMemorySegmentAddress",
    )?)
    .unwrap();

    let mut handles = Vec::new();
    let count = *CONNECTION_COUNT.get().unwrap();
    let is_running = Arc::new(AtomicBool::new(true));
    let request_counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(count + 1));
    let reqwest_client = build_request_client(CONNECTION_COUNT.get().unwrap().clone());
    for i in 0..count {
        // This is to initialize fory, create memory segment(Arena.shared) and other housekeeping work
        init_connection_fn(i as i32);
        let fory = init_fory()?;
        // Get the memory address where java will be writing
        let memory_segment_ptr = memory_segment_address_fn(i as i32) as usize;
        let c_barrier = barrier.clone();
        let is_running_clone = is_running.clone();
        let reqwest_client = reqwest_client.clone();
        let counter_clone = request_counter.clone();
        let fory_supplier_fn = fory_supplier_fn.clone();
        handles.push(tokio::spawn(async move {
            c_barrier.wait().await;
            while is_running_clone.load(Ordering::Relaxed) {
                // Java will update the memory with fory struct when this method is called in the memory segment which is the request that we want to execute
                // Fetch the request to be executed
                fory_supplier_fn(i as i32);
                let shared_data = unsafe {
                    let ptr = memory_segment_ptr as *const u8;
                    let length = std::ptr::read_unaligned(ptr as *const i32).to_le() as usize;
                    //println!("length: {}", length);
                    std::slice::from_raw_parts(ptr.add(4), length)
                };
                let fory_request = match fory.deserialize::<ForyRequest>(&shared_data) {
                    Ok(request) => request,
                    Err(e) => {
                        panic!("Deserialization issue: {:}", e);
                    }
                };
                // Take the absolute url and initialize the reqwest client with it
                // Java will take care of host, port and path and send it as absolute_url
                let absolute_url = fory_request.absolute_url;
                let method = reqwest_method(fory_request.method);
                let mut builder = reqwest_client.request(method, absolute_url);
                if !fory_request.body.is_empty() {
                    builder = builder.body(fory_request.body);
                }
                for (key, value) in &fory_request.headers {
                    builder = builder.header(key, value);
                }
                for (key, value) in &fory_request.query_params {
                    builder = builder.query(&[(key, value)]);
                }
                let request = builder.build().unwrap();
                let _response = match reqwest_client.execute(request).await {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("HTTP request error while executing the request {:?}", e);
                        continue;
                    }
                };
                //println!("Response code: {}", _response.status().as_u16());
                counter_clone.fetch_add(1, Ordering::Relaxed);

            }
        }));
    }

    let duration = Duration::from_secs(*RUN_DURATION.get().unwrap());
    let timer_is_running = is_running.clone();
    println!("All threads initialized. Starting execution...");
    barrier.wait().await;
    tokio::spawn(async move {
        tokio::time::sleep(duration).await;
        timer_is_running.store(false, Ordering::Relaxed);
        println!(
            "\nRun duration of {} elapsed. Signalling threads to stop.",
            format_duration(duration)
        );
    });
    let _ = futures::future::join_all(handles).await;
    println!("Total requests sent: {}", request_counter.load(Ordering::Relaxed));
    Ok(())
}
