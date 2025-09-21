#include "task.h"
#include "error.h"

/********************************************************************
 This sample schedules a task to start notepad.exe 1 minute from the
 time the task is registered.
********************************************************************/

#include <combaseapi.h>
#include <wtypes.h>
#define _WIN32_DCOM

// #include <comdef.h>
#include <stdio.h>
#include <wincred.h>
#include <windows.h>
//  Include the task header file.
#include <taskschd.h>
#pragma comment(lib, "taskschd.lib")
#pragma comment(lib, "comsupp.lib")
#pragma comment(lib, "credui.lib")

#define WORK_CLOCK_TASK_SERVICE_FOLDER L"\\"
#define WORK_CLOCK_TASK_NAME L"WorkClockEndNotificationTask"

WorkClockError task_manager_init(WorkClockTaskManager *task_manager) {
    task_manager->task_service = NULL;
    task_manager->task_folder = NULL;

    HRESULT hr = CoInitializeEx(NULL, COINIT_MULTITHREADED);
    ERROR_ASSERT(!FAILED(hr), E_TASK_INIT);

    hr = CoInitializeSecurity(NULL, -1, NULL, NULL, RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
                              RPC_C_IMP_LEVEL_IMPERSONATE, NULL, 0, NULL);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, CoUninitialize());

    ITaskService *task_service = NULL;
    hr = CoCreateInstance(&CLSID_TaskScheduler, NULL, CLSCTX_INPROC_SERVER, &IID_ITaskService,
                          (void **)&task_service);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, CoUninitialize());

    hr = task_service->lpVtbl->Connect(task_service, (VARIANT){}, (VARIANT){}, (VARIANT){},
                                       (VARIANT){});
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, {
        task_service->lpVtbl->Release(task_service);
        CoUninitialize();
    });

    ITaskFolder *task_folder = NULL;
    hr = task_service->lpVtbl->GetFolder(task_service, (BSTR)WORK_CLOCK_TASK_SERVICE_FOLDER,
                                         &task_folder);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, {
        task_service->lpVtbl->Release(task_service);
        CoUninitialize();
    });

    task_manager->task_service = task_service;
    task_manager->task_folder = task_folder;

    return E_SUCCESS;
}

WorkClockError task_manager_create_task(WorkClockTaskManager *task_manager) {
    ITaskService *task_service = task_manager->task_service;
    ITaskFolder *task_folder = task_manager->task_folder;
    ERROR_ASSERT(task_service != NULL && task_folder != NULL, E_TASK_INIT);

    ITaskDefinition *task = NULL;
    HRESULT hr = task_service->lpVtbl->NewTask(task_service, 0, &task);
    ERROR_ASSERT(!FAILED(hr), E_TASK_INIT);

    //  ------------------------------------------------------
    //  Create the principal for the task - these credentials
    //  are overwritten with the credentials passed to RegisterTaskDefinition
    IPrincipal *principal = NULL;
    hr = task->lpVtbl->get_Principal(task, &principal);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  Set up principal logon type to interactive logon
    hr = principal->lpVtbl->put_LogonType(principal, TASK_LOGON_INTERACTIVE_TOKEN);
    principal->lpVtbl->Release(principal);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  ------------------------------------------------------
    //  Create the settings for the task
    ITaskSettings *settings = NULL;
    hr = task->lpVtbl->get_Settings(task, &settings);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  Set setting values for the task.
    hr = settings->lpVtbl->put_StartWhenAvailable(settings, VARIANT_TRUE);
    settings->lpVtbl->Release(settings);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    // Set the idle settings for the task.
    IIdleSettings *idle_settings = NULL;
    hr = settings->lpVtbl->get_IdleSettings(settings, &idle_settings);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    hr = idle_settings->lpVtbl->put_WaitTimeout(idle_settings, L"PT5M");
    idle_settings->lpVtbl->Release(idle_settings);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  ------------------------------------------------------
    //  Get the trigger collection to insert the time trigger.
    ITriggerCollection *trigger_collection = NULL;
    hr = task->lpVtbl->get_Triggers(task, &trigger_collection);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  Add the time trigger to the task.
    ITrigger *trigger = NULL;
    hr = trigger_collection->lpVtbl->Create(trigger_collection, TASK_TRIGGER_TIME, &trigger);
    trigger_collection->lpVtbl->Release(trigger_collection);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    ITimeTrigger *time_trigger = NULL;
    hr = trigger->lpVtbl->QueryInterface(trigger, &IID_ITimeTrigger, (void **)&time_trigger);
    trigger->lpVtbl->Release(trigger);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    hr = time_trigger->lpVtbl->put_Id(time_trigger, (BSTR)L"Trigger1");
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, {
        time_trigger->lpVtbl->Release(time_trigger);
        task->lpVtbl->Release(task);
    });

    hr = time_trigger->lpVtbl->put_EndBoundary(time_trigger, (BSTR)L"2025-09-20T21:00:00-03:00");
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, {
        time_trigger->lpVtbl->Release(time_trigger);
        task->lpVtbl->Release(task);
    });

    //  Set the task to start at a certain time. The time
    //  format should be YYYY-MM-DDTHH:MM:SS(+-)(timezone).
    //  For example, the start boundary below
    //  is January 1st 2005 at 12:05
    hr = time_trigger->lpVtbl->put_StartBoundary(time_trigger, (BSTR)L"2025-09-20T20:45:00-03:00");
    time_trigger->lpVtbl->Release(time_trigger);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  ------------------------------------------------------
    //  Add an action to the task. This task will execute notepad.exe.
    IActionCollection *action_collection = NULL;
    //  Get the task action collection pointer.
    hr = task->lpVtbl->get_Actions(task, &action_collection);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  Create the action, specifying that it is an executable action.
    IAction *action = NULL;
    hr = action_collection->lpVtbl->Create(action_collection, TASK_ACTION_EXEC, &action);
    action_collection->lpVtbl->Release(action_collection);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    IExecAction *exec_action = NULL;
    //  QI for the executable task pointer.
    hr = action->lpVtbl->QueryInterface(action, &IID_IExecAction, (void **)&exec_action);
    action->lpVtbl->Release(action);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  Set the path of the executable to notepad.exe.
    hr = exec_action->lpVtbl->put_Path(exec_action, (BSTR)L"C:\\Windows\\System32\\Notepad.exe");
    exec_action->lpVtbl->Release(exec_action);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    //  ------------------------------------------------------
    //  Save the task in the root folder.
    // IRegisteredTask *registered_task = NULL;
    hr = task_folder->lpVtbl->RegisterTaskDefinition(
        task_folder, (BSTR)WORK_CLOCK_TASK_NAME, task, TASK_CREATE_OR_UPDATE, (VARIANT){},
        (VARIANT){}, TASK_LOGON_INTERACTIVE_TOKEN, (VARIANT){.bstrVal = L""}, NULL);
    ERROR_ASSERT_DO(!FAILED(hr), E_TASK_INIT, task->lpVtbl->Release(task));

    return E_SUCCESS;
}
