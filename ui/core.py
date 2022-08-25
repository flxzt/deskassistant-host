import os
import sys
import proc.core
from xdo import Xdo

def get_active_app_name() -> (str | None):
    match sys.platform:
        case "linux":
            match os.environ.get("XDG_SESSION_TYPE"):
                case "wayland":
                    # TODO
                    return None
                case "x11":
                    xdo_api = Xdo()

                    focused_win = xdo_api.get_focused_window_sane()
                    pid = xdo_api.get_pid_window(focused_win)
                    process = proc.core.Process.from_pid(pid)
                    if process == None:
                        return None

                    return process.exe_name
                case _:
                    return None
        case "win32":
            # TODO
            return None
        case _:
            return None
