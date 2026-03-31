#ifndef WORKCLOCK_ERROR_H
#define WORKCLOCK_ERROR_H

#define ERROR_ASSERT(condition, error)                                                             \
    if (!(condition)) {                                                                            \
        return error;                                                                              \
    }

#define ERROR_ASSERT_DO(condition, error, action)                                                  \
    if (!(condition)) {                                                                            \
        action;                                                                                    \
        return error;                                                                              \
    }

typedef enum {
    E_SUCCESS,
    E_TASK_INIT,
} WorkClockError;

#endif
