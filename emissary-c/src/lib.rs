//! C Foreign Function Interface (FFI) bindings for the Emissary I2P router.
//!
//! This crate provides a minimal C API for embedding an I2P router in C applications.
//! The API focuses on essential lifecycle management and SAMv3 bridge access.
//!
//! # Safety
//!
//! This FFI layer is designed with safety in mind:
//! - All functions check for null pointers before dereferencing
//! - Panic boundaries are established using `catch_unwind`
//! - Memory management follows Box allocation patterns
//! - Thread safety is provided through internal synchronization
//!
//! # Usage Pattern
//!
//! ```c
//! emissary_router_t* router = emissary_init();
//! if (!router) return -1;
//!
//! int result = emissary_start(router);
//! if (result < 0) {
//!     emissary_destroy(router);
//!     return result;
//! }
//!
//! // Wait for router to become operational
//! while (emissary_get_status(router) == EMISSARY_STATUS_STARTING) {
//!     sleep(1);
//! }
//!
//! // Use SAMv3 bridge if available
//! if (emissary_sam_available(router)) {
//!     int port = emissary_get_sam_tcp_port(router);
//!     // Connect to 127.0.0.1:port for I2P API
//! }
//!
//! emissary_stop(router);
//! emissary_destroy(router);
//! ```

use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
    sync::{Arc, Mutex, RwLock},
};

use emissary_core::{
    events::EventSubscriber,
    router::RouterBuilder,
    Config, SamConfig,
};
use emissary_util::runtime::tokio::Runtime;
use tokio::sync::mpsc;

// ============================================================================
// Internal Types and State Management
// ============================================================================

/// Internal router state representation
enum RouterState {
    /// Router has been created but not started
    Stopped,
    /// Router is in the process of starting up
    Starting,
    /// Router is running and operational
    Running {
        _events: EventSubscriber,
        shutdown_tx: mpsc::Sender<()>,
        _runtime: Arc<tokio::runtime::Runtime>,
    },
    /// Router is in the process of shutting down
    Stopping,
    /// Router encountered an error
    Error,
}

/// Opaque router handle - represents the C-visible router instance
pub struct EmissaryRouter {
    /// Router state protected by mutex for thread safety
    state: Mutex<RouterState>,
    /// Router configuration
    config: RwLock<Config>,
    /// SAMv3 port information
    sam_tcp_port: RwLock<Option<u16>>,
    sam_udp_port: RwLock<Option<u16>>,
}

impl EmissaryRouter {
    /// Create new router instance with default configuration
    fn new() -> Self {
        // Create default configuration with SAMv3 enabled
        let mut config = Config::default();
        
        // Enable SAMv3 with default ports (0 = random port assignment)
        config.samv3_config = Some(SamConfig {
            tcp_port: 0, // Will be assigned by OS
            udp_port: 0, // Will be assigned by OS
            host: "127.0.0.1".to_string(),
        });
        
        // Disable transit tunnels for minimal resource usage
        config.transit = None;
        
        // Enable insecure tunnels for faster startup in development
        config.insecure_tunnels = true;

        Self {
            state: Mutex::new(RouterState::Stopped),
            config: RwLock::new(config),
            sam_tcp_port: RwLock::new(None),
            sam_udp_port: RwLock::new(None),
        }
    }
}

// ============================================================================
// Error Code Constants (matching header file)
// ============================================================================

const EMISSARY_SUCCESS: i32 = 0;
const EMISSARY_ERROR_GENERIC: i32 = -1;
const EMISSARY_ERROR_INVALID_PARAM: i32 = -2;
const EMISSARY_ERROR_ALREADY_STARTED: i32 = -4;
const EMISSARY_ERROR_NOT_STARTED: i32 = -5;
const EMISSARY_ERROR_NETWORK: i32 = -6;
const EMISSARY_ERROR_RESOURCE: i32 = -7;

// Status constants
const EMISSARY_STATUS_STOPPED: i32 = 0;
const EMISSARY_STATUS_STARTING: i32 = 1;
const EMISSARY_STATUS_RUNNING: i32 = 2;
const EMISSARY_STATUS_STOPPING: i32 = 3;
const EMISSARY_STATUS_ERROR: i32 = 4;

// ============================================================================
// Core Lifecycle Functions
// ============================================================================

/// Initialize a new I2P router instance
#[no_mangle]
pub extern "C" fn emissary_init() -> *mut EmissaryRouter {
    // Catch any panics and return NULL on failure
    match catch_unwind(|| {
        Box::into_raw(Box::new(EmissaryRouter::new()))
    }) {
        Ok(router_ptr) => router_ptr,
        Err(_) => ptr::null_mut(),
    }
}

/// Start the I2P router and begin network operations
#[no_mangle]
pub extern "C" fn emissary_start(router_ptr: *mut EmissaryRouter) -> i32 {
    if router_ptr.is_null() {
        return EMISSARY_ERROR_INVALID_PARAM;
    }

    // Safety: We've checked for null pointer
    let router = unsafe { &*router_ptr };

    match catch_unwind(AssertUnwindSafe(|| {
        let mut state_guard = match router.state.lock() {
            Ok(guard) => guard,
            Err(_) => return EMISSARY_ERROR_GENERIC,
        };

        match *state_guard {
            RouterState::Running { .. } => return EMISSARY_ERROR_ALREADY_STARTED,
            RouterState::Starting => return EMISSARY_ERROR_ALREADY_STARTED,
            RouterState::Stopping => return EMISSARY_ERROR_GENERIC,
            RouterState::Error => return EMISSARY_ERROR_GENERIC,
            RouterState::Stopped => {
                // Proceed with startup
            }
        }

        // Update state to starting
        *state_guard = RouterState::Starting;
        drop(state_guard); // Release lock during async operations

        // Create Tokio runtime for the router
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build() 
        {
            Ok(rt) => Arc::new(rt),
            Err(_) => {
                // Reset state on failure
                if let Ok(mut guard) = router.state.lock() {
                    *guard = RouterState::Error;
                }
                return EMISSARY_ERROR_RESOURCE;
            }
        };

        // Clone runtime for the blocking operation
        let _rt_clone = Arc::clone(&rt);
        
        // Start router asynchronously with improved error handling
        let start_result = rt.block_on(async move {
            // Read configuration
            let config = match router.config.read() {
                Ok(guard) => (*guard).clone(),
                Err(_) => return Err(EMISSARY_ERROR_GENERIC),
            };

            // Create router builder and build router
            let builder = RouterBuilder::<Runtime>::new(config);
            
            match builder.build().await {
                Ok((emissary_router, events, _router_info)) => {
                    // Extract SAMv3 port information from the router
                    let protocol_info = emissary_router.protocol_address_info();
                    
                    if let Some(sam_tcp) = protocol_info.sam_tcp {
                        if let Ok(mut port_guard) = router.sam_tcp_port.write() {
                            *port_guard = Some(sam_tcp.port());
                        }
                    }
                    
                    if let Some(sam_udp) = protocol_info.sam_udp {
                        if let Ok(mut port_guard) = router.sam_udp_port.write() {
                            *port_guard = Some(sam_udp.port());
                        }
                    }

                    Ok((emissary_router, events))
                }
                Err(_e) => {
                    // Log error details for debugging (in development builds)
                    #[cfg(debug_assertions)]
                    eprintln!("Router startup failed: {:?}", _e);
                    
                    Err(EMISSARY_ERROR_NETWORK)
                },
            }
        });

        match start_result {
            Ok((emissary_router, events)) => {
                // Create shutdown channel
                let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
                
                // Spawn router task in the runtime
                let runtime_clone = Arc::clone(&rt);
                runtime_clone.spawn(async move {
                    tokio::select! {
                        _ = emissary_router => {
                            // Router completed
                        }
                        _ = shutdown_rx.recv() => {
                            // Shutdown requested - router will be dropped
                        }
                    }
                });

                // Update state to running
                if let Ok(mut guard) = router.state.lock() {
                    *guard = RouterState::Running {
                        _events: events,
                        shutdown_tx,
                        _runtime: runtime_clone,
                    };
                }

                EMISSARY_SUCCESS
            }
            Err(error_code) => {
                // Reset state on failure
                if let Ok(mut guard) = router.state.lock() {
                    *guard = RouterState::Error;
                }
                error_code
            }
        }
    })) {
        Ok(result) => result,
        Err(_) => {
            // Panic occurred, reset state
            if let Ok(mut guard) = router.state.lock() {
                *guard = RouterState::Error;
            }
            EMISSARY_ERROR_GENERIC
        }
    }
}

/// Stop the I2P router and cease network operations
#[no_mangle]
pub extern "C" fn emissary_stop(router_ptr: *mut EmissaryRouter) -> i32 {
    if router_ptr.is_null() {
        return EMISSARY_ERROR_INVALID_PARAM;
    }

    // Safety: We've checked for null pointer
    let router = unsafe { &*router_ptr };

    match catch_unwind(AssertUnwindSafe(|| {
        let mut state_guard = match router.state.lock() {
            Ok(guard) => guard,
            Err(_) => return EMISSARY_ERROR_GENERIC,
        };

        match std::mem::replace(&mut *state_guard, RouterState::Stopping) {
            RouterState::Running { shutdown_tx, .. } => {
                drop(state_guard); // Release lock during shutdown

                // Signal shutdown
                let _ = shutdown_tx.try_send(());

                // Update state
                if let Ok(mut guard) = router.state.lock() {
                    *guard = RouterState::Stopped;
                }

                EMISSARY_SUCCESS
            }
            RouterState::Stopped => {
                // Reset state and return error
                *state_guard = RouterState::Stopped;
                EMISSARY_ERROR_NOT_STARTED
            }
            RouterState::Starting => {
                // Reset state and return error
                *state_guard = RouterState::Starting;
                EMISSARY_ERROR_NOT_STARTED
            }
            RouterState::Stopping => {
                // Already stopping
                *state_guard = RouterState::Stopping;
                EMISSARY_SUCCESS
            }
            RouterState::Error => {
                // Reset state and return error
                *state_guard = RouterState::Error;
                EMISSARY_ERROR_GENERIC
            }
        }
    })) {
        Ok(result) => result,
        Err(_) => EMISSARY_ERROR_GENERIC,
    }
}

/// Destroy router instance and free all associated resources
#[no_mangle]
pub extern "C" fn emissary_destroy(router_ptr: *mut EmissaryRouter) {
    if router_ptr.is_null() {
        return;
    }

    // First attempt to stop the router if it's running
    let _ = emissary_stop(router_ptr);

    // Convert back to Box and drop to free memory
    // Safety: This pointer came from Box::into_raw in emissary_init
    let _ = catch_unwind(|| unsafe {
        let _ = Box::from_raw(router_ptr);
    });
}

// ============================================================================
// Status and Information Functions
// ============================================================================

/// Get current router operational status
#[no_mangle]
pub extern "C" fn emissary_get_status(router_ptr: *mut EmissaryRouter) -> i32 {
    if router_ptr.is_null() {
        return EMISSARY_ERROR_INVALID_PARAM;
    }

    // Safety: We've checked for null pointer
    let router = unsafe { &*router_ptr };

    match catch_unwind(AssertUnwindSafe(|| {
        match router.state.lock() {
            Ok(guard) => match &*guard {
                RouterState::Stopped => EMISSARY_STATUS_STOPPED,
                RouterState::Starting => EMISSARY_STATUS_STARTING,
                RouterState::Running { .. } => EMISSARY_STATUS_RUNNING,
                RouterState::Stopping => EMISSARY_STATUS_STOPPING,
                RouterState::Error => EMISSARY_STATUS_ERROR,
            },
            Err(_) => EMISSARY_ERROR_GENERIC,
        }
    })) {
        Ok(status) => status,
        Err(_) => EMISSARY_ERROR_GENERIC,
    }
}

/// Check if SAMv3 API bridge is available and operational
#[no_mangle]
pub extern "C" fn emissary_sam_available(router_ptr: *mut EmissaryRouter) -> i32 {
    if router_ptr.is_null() {
        return EMISSARY_ERROR_INVALID_PARAM;
    }

    // Safety: We've checked for null pointer
    let router = unsafe { &*router_ptr };

    match catch_unwind(AssertUnwindSafe(|| {
        match router.state.lock() {
            Ok(guard) => match &*guard {
                RouterState::Running { .. } => {
                    // Check if we have SAM port information
                    match (router.sam_tcp_port.read(), router.sam_udp_port.read()) {
                        (Ok(tcp_guard), Ok(udp_guard)) => {
                            if tcp_guard.is_some() && udp_guard.is_some() {
                                1 // SAM is available
                            } else {
                                0 // SAM is not available
                            }
                        }
                        _ => 0, // Error reading port info
                    }
                }
                _ => 0, // Router not running
            },
            Err(_) => EMISSARY_ERROR_GENERIC,
        }
    })) {
        Ok(result) => result,
        Err(_) => EMISSARY_ERROR_GENERIC,
    }
}

/// Get SAMv3 TCP port number
#[no_mangle]
pub extern "C" fn emissary_get_sam_tcp_port(router_ptr: *mut EmissaryRouter) -> i32 {
    if router_ptr.is_null() {
        return EMISSARY_ERROR_INVALID_PARAM;
    }

    // Safety: We've checked for null pointer
    let router = unsafe { &*router_ptr };

    match catch_unwind(AssertUnwindSafe(|| {
        match router.sam_tcp_port.read() {
            Ok(guard) => match *guard {
                Some(port) => port as i32,
                None => 0, // Not available
            },
            Err(_) => EMISSARY_ERROR_GENERIC,
        }
    })) {
        Ok(result) => result,
        Err(_) => EMISSARY_ERROR_GENERIC,
    }
}

/// Get SAMv3 UDP port number
#[no_mangle]
pub extern "C" fn emissary_get_sam_udp_port(router_ptr: *mut EmissaryRouter) -> i32 {
    if router_ptr.is_null() {
        return EMISSARY_ERROR_INVALID_PARAM;
    }

    // Safety: We've checked for null pointer
    let router = unsafe { &*router_ptr };

    match catch_unwind(AssertUnwindSafe(|| {
        match router.sam_udp_port.read() {
            Ok(guard) => match *guard {
                Some(port) => port as i32,
                None => 0, // Not available
            },
            Err(_) => EMISSARY_ERROR_GENERIC,
        }
    })) {
        Ok(result) => result,
        Err(_) => EMISSARY_ERROR_GENERIC,
    }
}
