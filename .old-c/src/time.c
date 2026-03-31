#include "time.h"
#include <windows.h>

#define FILETIME_TO_UNIX_EPOCH 116444736000000000ULL

void time_get_time(WorkClockTime *time) {
    FILETIME file_time;
    GetSystemTimeAsFileTime(&file_time);

    ULARGE_INTEGER uli_time = {.LowPart = file_time.dwLowDateTime,
                               .HighPart = file_time.dwHighDateTime};

    time->unix_time = (time_t)((uli_time.QuadPart - FILETIME_TO_UNIX_EPOCH) / 10000000ULL);
}