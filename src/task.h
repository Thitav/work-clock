#ifndef WORKCLOCK_TASK_H
#define WORKCLOCK_TASK_H

#include "error.h"
#include <taskschd.h>

typedef struct {
    ITaskService *task_service;
    ITaskFolder *task_folder;
} WorkClockTaskManager;

WorkClockError task_manager_init(WorkClockTaskManager *task_manager);
WorkClockError task_manager_create_task(WorkClockTaskManager *task_manager);

#endif
