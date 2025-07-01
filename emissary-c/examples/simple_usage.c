/*
 * Simple example demonstrating the minimal I2P router lifecycle using emissary-c.
 * 
 * This example shows how to:
 * 1. Initialize an I2P router instance
 * 2. Start the router and wait for it to become operational
 * 3. Access the SAMv3 API bridge ports
 * 4. Stop the router gracefully
 * 5. Clean up resources
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>  // for sleep()
#include "../emissary-c.h"

int main(void) {
    printf("Emissary I2P Router C API Example\n");
    printf("==================================\n\n");

    // Step 1: Initialize the router
    printf("1. Initializing I2P router...\n");
    emissary_router_t* router = emissary_init();
    if (router == NULL) {
        fprintf(stderr, "Error: Failed to initialize router\n");
        return EXIT_FAILURE;
    }
    printf("   Router initialized successfully\n\n");

    // Step 2: Start the router
    printf("2. Starting I2P router...\n");
    int result = emissary_start(router);
    if (result != EMISSARY_SUCCESS) {
        fprintf(stderr, "Error: Failed to start router (code: %d)\n", result);
        emissary_destroy(router);
        return EXIT_FAILURE;
    }
    printf("   Router startup initiated\n\n");

    // Step 3: Wait for router to become operational
    printf("3. Waiting for router to become operational...\n");
    int status;
    int attempts = 0;
    const int max_attempts = 30; // 30 seconds timeout
    
    do {
        status = emissary_get_status(router);
        if (status < 0) {
            fprintf(stderr, "Error: Failed to get router status (code: %d)\n", status);
            emissary_destroy(router);
            return EXIT_FAILURE;
        }
        
        switch (status) {
            case EMISSARY_STATUS_STARTING:
                printf("   Router is starting... (%d/%d)\n", attempts + 1, max_attempts);
                break;
            case EMISSARY_STATUS_RUNNING:
                printf("   Router is now running!\n");
                break;
            case EMISSARY_STATUS_ERROR:
                fprintf(stderr, "Error: Router entered error state\n");
                emissary_destroy(router);
                return EXIT_FAILURE;
            default:
                fprintf(stderr, "Error: Unexpected router status: %d\n", status);
                emissary_destroy(router);
                return EXIT_FAILURE;
        }
        
        if (status != EMISSARY_STATUS_RUNNING) {
            sleep(1); // Wait 1 second before checking again
            attempts++;
        }
    } while (status != EMISSARY_STATUS_RUNNING && attempts < max_attempts);

    if (status != EMISSARY_STATUS_RUNNING) {
        fprintf(stderr, "Error: Router failed to start within timeout period\n");
        emissary_destroy(router);
        return EXIT_FAILURE;
    }
    printf("\n");

    // Step 4: Check SAMv3 availability and get port information
    printf("4. Checking SAMv3 API bridge...\n");
    
    int sam_available = emissary_sam_available(router);
    if (sam_available < 0) {
        fprintf(stderr, "Error: Failed to check SAMv3 availability (code: %d)\n", sam_available);
    } else if (sam_available == 0) {
        printf("   SAMv3 API bridge is not available\n");
    } else {
        printf("   SAMv3 API bridge is available\n");
        
        // Get SAMv3 port numbers
        int tcp_port = emissary_get_sam_tcp_port(router);
        int udp_port = emissary_get_sam_udp_port(router);
        
        if (tcp_port > 0 && udp_port > 0) {
            printf("   SAMv3 TCP port: %d\n", tcp_port);
            printf("   SAMv3 UDP port: %d\n", udp_port);
            printf("   Applications can connect to 127.0.0.1:%d for SAMv3 API\n", tcp_port);
        } else {
            printf("   Port information not available\n");
        }
    }
    printf("\n");

    // Step 5: Let the router run for a short time
    printf("5. Router is operational. Running for 10 seconds...\n");
    printf("   (In a real application, this is where your I2P operations would occur)\n");
    sleep(10);
    printf("\n");

    // Step 6: Stop the router
    printf("6. Stopping I2P router...\n");
    result = emissary_stop(router);
    if (result != EMISSARY_SUCCESS) {
        fprintf(stderr, "Warning: Failed to stop router gracefully (code: %d)\n", result);
    } else {
        printf("   Router shutdown initiated\n");
        
        // Wait for shutdown to complete
        attempts = 0;
        do {
            status = emissary_get_status(router);
            if (status == EMISSARY_STATUS_STOPPING) {
                printf("   Router is stopping... (%d/10)\n", attempts + 1);
                sleep(1);
                attempts++;
            }
        } while (status == EMISSARY_STATUS_STOPPING && attempts < 10);
        
        if (status == EMISSARY_STATUS_STOPPED) {
            printf("   Router stopped successfully\n");
        } else {
            printf("   Router stop status: %d\n", status);
        }
    }
    printf("\n");

    // Step 7: Clean up resources
    printf("7. Cleaning up resources...\n");
    emissary_destroy(router);
    printf("   Router resources freed\n\n");

    printf("Example completed successfully!\n");
    return EXIT_SUCCESS;
}
