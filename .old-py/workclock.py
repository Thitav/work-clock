#!/usr/bin/python3

import time
import sys
import pickle
import datetime
# import pathlib

STORAGE_FILE_PREFIX = "wcsession-"

program_start_time = time.time()

class WorkSession:
    def __init__(self, duration: float | None):
        if duration <= 0:
            raise Exception("Negative duration")
            
        self.duration = duration
        self.pauses: list[list[float, float | None]] = []
        self.start = program_start_time
        self.finished_at = None

    def pause(self):
        if self.is_finished():
            raise Exception("Session already finished")
        if self.is_paused():
            raise Exception("Session is already paused")

        self.pauses.append([program_start_time, None])

    def unpause(self):
        if self.is_finished():
            raise Exception("Session already finished")
        if not self.is_paused():
            raise Exception("Session is not paused")

        pause_duration = program_start_time - self.pauses[-1][0]
        self.pauses[-1][1] = pause_duration

    def finish(self, bypass_duration = False):
        if self.is_finished():
            raise Exception("Session already finished")
        
        if not bypass_duration and self.is_limited():
            if self.remaining() > 0:
                raise Exception("Session cannot be finished yet")

        if self.is_paused():
            self.unpause()
        self.finished_at = program_start_time
        self.duration = self.real_duration()

    def is_paused(self):
        return len(self.pauses) > 0 and self.pauses[-1][1] == None

    def is_limited(self):
        return self.duration != None

    def is_finished(self):
        return self.finished_at != None

    def total_paused_time(self):
        total_time = 0.0
        for start, duration in self.pauses:
            if duration == 0.0:
                duration = program_start_time - start
            total_time += duration
        return total_time
    
    def total_paused_time_datetime(self):
        return datetime.timedelta(seconds=self.total_paused_time())

    def pauses_iter(self):
        for pause in self.pauses:
            yield pause 
            
    def pauses_iter_datetime(self):
        for start, duration in self.pauses:
            yield [
                datetime.datetime.fromtimestamp(start),
                datetime.timedelta(seconds=duration if duration != None else (program_start_time - start))
            ]

    def eta(self):
        if not self.is_limited():
            return None
        return self.start + self.duration + self.total_paused_time()        

    def eta_datetime(self):
        timestamp = self.eta()
        if timestamp == None:
            return None
        return datetime.datetime.fromtimestamp(timestamp)

    def remaining(self):
        if not self.is_limited():
            return None
        
        return self.duration - self.real_duration()

    def remaining_datetime(self):
        timestamp = self.remaining()
        if timestamp == None:
            return None
        return datetime.timedelta(seconds=timestamp)

    def real_duration(self):
        if self.is_finished():
            return self.duration
        return program_start_time - self.start - self.total_paused_time()

    def real_duration_datetime(self):
        return datetime.timedelta(seconds=self.real_duration())

    def finished_at_datetime(self):
        if not self.is_finished():
            return None
        return datetime.datetime.fromtimestamp(self.finished_at)

    @staticmethod
    def load(start_time: float | None):
        filename = f"{STORAGE_FILE_PREFIX}current.pkl" if start_time == None else f"{STORAGE_FILE_PREFIX}{str(int(start_time * 1000))}.pkl"
        with open(filename, "rb") as f:
            session: WorkSession = pickle.load(f)
            f.close()
        return session

    def save(self, is_current: bool):
        filename = f"{STORAGE_FILE_PREFIX}current.pkl" if is_current else f"{STORAGE_FILE_PREFIX}{str(int(self.start * 1000))}.pkl"
        with open(filename, "wb") as f:
            pickle.dump(self, f)
            f.close()
            
    @staticmethod
    def clear_current():
        with open(f"{STORAGE_FILE_PREFIX}current.pkl", "wb") as f:
            f.write(b"")
            f.close()


def print_session_status(session: WorkSession):
    if session.is_finished():
        print("Session finished")
        print(f"Finished at {session.finished_at_datetime().strftime('%Y-%m-%d %H:%M:%S')}")
    else:
        print("Session paused" if session.is_paused() else "Session running")
        if session.is_limited():
            print(f"ETA: {session.eta_datetime().strftime('%Y-%m-%d %H:%M:%S')}")

            remaining = session.remaining_datetime()
            if remaining.total_seconds() >= 0.0:
                print(f"Time remaining: {format_timedelta(remaining)}")
            else:
                print(f"Overtime: {format_timedelta(datetime.timedelta(seconds=-remaining.total_seconds()))}")
    print(f"Worked for: {format_timedelta(session.real_duration_datetime())}")

def format_timedelta(td: datetime.timedelta):
    if td.total_seconds() <= 0:
        return "0s"

    hours = td.seconds // 3600
    minutes = (td.seconds % 3600) // 60
    seconds = td.seconds % 60

    s = ""
    if td.days > 0:
        s += f"{td.days}d "
    if hours > 0:
        s += f"{hours}h"
    if minutes > 0:
        s += f"{minutes}m"
    if seconds > 0:
        s += f"{seconds}s"
    return s.strip()


mode = sys.argv[1].lower().strip()

if mode == "start" or mode == "s":
    duration = int(sys.argv[2]) * 60

    session = WorkSession(duration if duration != 0 else None)
    session.save(True)

    print_session_status(session)

elif mode == "check" or mode == "c":
    session = WorkSession.load(None)
    print_session_status(session)

elif mode == "pause" or mode == "p":
    session = WorkSession.load(None)

    session.pause()
    session.save(True)
    print("Clock paused")

elif mode == "unpause" or mode == "u":
    session = WorkSession.load(None)

    session.unpause()
    session.save(True)
    print("Clock unpaused")

elif mode == "log" or mode == "l":
    session = WorkSession.load(None)

    print(f"Total paused time: {format_timedelta(session.total_paused_time_datetime())}")

    for pause_date, pause_duration in session.pauses_iter_datetime():
        print(
            f"- {pause_date.strftime('%Y-%m-%d %H:%M:%S')}: {format_timedelta(pause_duration)}"
        )
        
elif mode == "finish" or mode == "f":
    session = WorkSession.load(None)
    session.finish()
    session.save(False)
    WorkSession.clear_current()
    
# elif mode == "migrate":
#     import pathlib
#     path = pathlib.Path(".").resolve()
#     for f in path.iterdir():
#         if not f.name.startswith(STORAGE_FILE_PREFIX):
#             continue
#         old_session = pickle.load(f.open(mode = "rb"))
#         new_session = WorkSession(old_session.duration)
#         new_session.start = old_session.start
#         new_session.pauses = old_session.pauses
#        
#         if "current" in f.name:    
#             new_session.save(True)
#         else:
#             new_session.finish()
#             new_session.save(False)
           
else:
    print("Invalid option")
