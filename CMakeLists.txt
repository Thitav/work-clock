cmake_minimum_required(VERSION 3.10.0)
project(work-clock VERSION 0.1.0 LANGUAGES C)

add_executable(work-clock src/main.c src/task.c src/task.h src/time.c src/time.h src/error.h)

include(CTest)
enable_testing()

set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
