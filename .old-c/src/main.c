// #define _WIN32_DCOM

#include "task.h"
#include "time.h"
#include <stdio.h>

int main() {
    // WorkClockTaskManager tm;

    // if (task_manager_init(&tm) != E_SUCCESS) {
    //     puts("error1");
    // }

    // if (task_manager_create_task(&tm) != E_SUCCESS) {
    //     puts("error2");
    // }

    WorkClockTime wt;
    time_get_time(&wt);
    printf("%lld %lld\n", wt.unix_time, time(NULL));

    return 0;
}