#ifndef EMISSARY_C_H
#define EMISSARY_C_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ====================================================================== */
/*                              Type Definitions                         */
/* ====================================================================== */

/**
 * Opaque handle to an I2P router instance.
 * 
 * This handle represents a running or stopped I2P router instance.
 * It must be created with emissary_init() and destroyed with emissary_destroy().
 */
typedef struct emissary_router emissary_router_t;

/* ====================================================================== */
/*                              Error Codes                              */
/* ====================================================================== */

/** Success - operation completed successfully */
#define EMISSARY_SUCCESS                0

/** Generic error - unspecified failure */
#define EMISSARY_ERROR_GENERIC         -1

/** Invalid parameter - NULL pointer or invalid value */
#define EMISSARY_ERROR_INVALID_PARAM   -2

/** Router not initialized - handle is invalid or destroyed */
#define EMISSARY_ERROR_NOT_INITIALIZED -3

/** Already started - router is already running */
#define EMISSARY_ERROR_ALREADY_STARTED -4

/** Not started - router is not currently running */
#define EMISSARY_ERROR_NOT_STARTED     -5

/** Network error - failed to bind ports or establish connections */
#define EMISSARY_ERROR_NETWORK         -6

/** Resource error - insufficient memory or system resources */
#define EMISSARY_ERROR_RESOURCE        -7

/** SAM bridge unavailable - SAMv3 API bridge is not enabled or failed */
#define EMISSARY_ERROR_SAM_UNAVAILABLE -8

/* ====================================================================== */
/*                              Status Codes                             */
/* ====================================================================== */

/** Router is stopped */
#define EMISSARY_STATUS_STOPPED        0

/** Router is starting up */
#define EMISSARY_STATUS_STARTING       1

/** Router is running and ready */
#define EMISSARY_STATUS_RUNNING        2

/** Router is shutting down */
#define EMISSARY_STATUS_STOPPING       3

/** Router is in an error state */
#define EMISSARY_STATUS_ERROR          4

/* ====================================================================== */
/*                            Core Lifecycle Functions                   */
/* ====================================================================== */

/**
 * Initialize a new I2P router instance.
 * 
 * Creates and configures an I2P router with sensible defaults:
 * - NTCP2 transport enabled on a random port (unpublished)
 * - Transit tunnels disabled for minimal resource usage
 * - SAMv3 API bridge enabled on random ports
 * - Local data directory in system temp location
 * - Insecure tunnels enabled for faster startup
 * 
 * The router is created in a stopped state. Call emissary_start() to begin
 * I2P network operations.
 * 
 * @return Opaque handle to router instance, or NULL on failure.
 *         Caller must eventually pass handle to emissary_destroy().
 * 
 * Thread Safety: This function is thread-safe and may be called concurrently.
 */
emissary_router_t* emissary_init(void);

/**
 * Start the I2P router and begin network operations.
 * 
 * Starts all configured subsystems:
 * - Transport protocols (NTCP2/SSU2)
 * - Tunnel management
 * - Network database
 * - SAMv3 API bridge
 * 
 * This function is non-blocking and returns immediately after initiating
 * startup. Use emissary_get_status() to check when the router is fully running.
 * 
 * @param router Handle to router instance from emissary_init()
 * @return EMISSARY_SUCCESS on successful startup initiation
 *         EMISSARY_ERROR_INVALID_PARAM if router is NULL
 *         EMISSARY_ERROR_NOT_INITIALIZED if router was destroyed
 *         EMISSARY_ERROR_ALREADY_STARTED if router is already running
 *         EMISSARY_ERROR_NETWORK on network configuration failure
 *         EMISSARY_ERROR_RESOURCE on insufficient system resources
 * 
 * Thread Safety: This function is NOT thread-safe. Do not call concurrently
 *                on the same router handle.
 */
int emissary_start(emissary_router_t* router);

/**
 * Stop the I2P router and cease network operations.
 * 
 * Initiates graceful shutdown of all router subsystems:
 * - Closes active tunnels and connections
 * - Saves network database to disk
 * - Stops SAMv3 API bridge
 * - Releases network ports
 * 
 * First call initiates graceful shutdown. Subsequent calls force immediate
 * shutdown, cancelling graceful procedures.
 * 
 * This function is non-blocking and returns immediately after initiating
 * shutdown. Use emissary_get_status() to check when shutdown is complete.
 * 
 * @param router Handle to router instance
 * @return EMISSARY_SUCCESS on successful shutdown initiation
 *         EMISSARY_ERROR_INVALID_PARAM if router is NULL
 *         EMISSARY_ERROR_NOT_INITIALIZED if router was destroyed
 *         EMISSARY_ERROR_NOT_STARTED if router is not currently running
 * 
 * Thread Safety: This function is NOT thread-safe. Do not call concurrently
 *                on the same router handle.
 */
int emissary_stop(emissary_router_t* router);

/**
 * Destroy router instance and free all associated resources.
 * 
 * If the router is currently running, this function will stop it first
 * using emergency shutdown procedures (equivalent to calling emissary_stop()
 * twice).
 * 
 * After this function returns, the router handle becomes invalid and must
 * not be used in any subsequent calls.
 * 
 * @param router Handle to router instance, may be NULL (no-op)
 * 
 * Thread Safety: This function is NOT thread-safe. Do not call concurrently
 *                on the same router handle. Ensure no other operations are
 *                active on this handle.
 */
void emissary_destroy(emissary_router_t* router);

/* ====================================================================== */
/*                              Status Functions                         */
/* ====================================================================== */

/**
 * Get current router operational status.
 * 
 * @param router Handle to router instance
 * @return One of EMISSARY_STATUS_* constants, or negative error code:
 *         EMISSARY_ERROR_INVALID_PARAM if router is NULL
 *         EMISSARY_ERROR_NOT_INITIALIZED if router was destroyed
 * 
 * Thread Safety: This function is thread-safe and may be called concurrently.
 */
int emissary_get_status(emissary_router_t* router);

/**
 * Check if SAMv3 API bridge is available and operational.
 * 
 * @param router Handle to router instance
 * @return 1 if SAMv3 bridge is available and router is running
 *         0 if SAMv3 bridge is unavailable or router is stopped
 *         Negative error code on failure:
 *         EMISSARY_ERROR_INVALID_PARAM if router is NULL
 *         EMISSARY_ERROR_NOT_INITIALIZED if router was destroyed
 * 
 * Thread Safety: This function is thread-safe and may be called concurrently.
 */
int emissary_sam_available(emissary_router_t* router);

/* ====================================================================== */
/*                          SAMv3 Bridge Functions                       */
/* ====================================================================== */

/**
 * Get SAMv3 TCP port number.
 * 
 * Returns the port number where the SAMv3 TCP server is listening.
 * Applications can connect to 127.0.0.1:<port> to use the SAMv3 API.
 * 
 * @param router Handle to router instance
 * @return Port number (1-65535) if available
 *         0 if SAMv3 is not available or router is not running
 *         Negative error code on failure:
 *         EMISSARY_ERROR_INVALID_PARAM if router is NULL
 *         EMISSARY_ERROR_NOT_INITIALIZED if router was destroyed
 * 
 * Thread Safety: This function is thread-safe and may be called concurrently.
 */
int emissary_get_sam_tcp_port(emissary_router_t* router);

/**
 * Get SAMv3 UDP port number.
 * 
 * Returns the port number where the SAMv3 UDP server is listening.
 * Applications can send datagrams to 127.0.0.1:<port> for datagram-style
 * I2P communication.
 * 
 * @param router Handle to router instance
 * @return Port number (1-65535) if available
 *         0 if SAMv3 is not available or router is not running
 *         Negative error code on failure:
 *         EMISSARY_ERROR_INVALID_PARAM if router is NULL
 *         EMISSARY_ERROR_NOT_INITIALIZED if router was destroyed
 * 
 * Thread Safety: This function is thread-safe and may be called concurrently.
 */
int emissary_get_sam_udp_port(emissary_router_t* router);

#ifdef __cplusplus
}
#endif

#endif /* EMISSARY_C_H */
