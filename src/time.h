#ifndef WORK_CLOCK_TIME_H
#define WORK_CLOCK_TIME_H

#include <time.h>

typedef struct {
    time_t unix_time;
} WorkClockTime;

void time_get_time(WorkClockTime *time);

#endif
