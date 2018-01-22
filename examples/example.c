#include <stdio.h>
#include <stdint.h>

int32_t battery(uint8_t *bus_id, uint32_t event_type, uint32_t event_data) {
    printf("bus_id: %s\nevent_type: %d\nevent_data: %d\n",
            bus_id, event_type, event_data);
    return -1;
}
