#!/usr/bin/env python3
"""Exercise the packaged UI on a private X11 display; never touch user data.

Needs Xvfb, Openbox, xcompmgr, xdotool, xwininfo, xprop, ImageMagick, and libX11/libXext.
Usage: python3 scripts/smoke-linux.py [binary] [output-directory]
"""
import ctypes
import ctypes.util
import os
import secrets
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def main():
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "src-tauri/target/release/lm-overlay").resolve()
    output = Path(sys.argv[2] if len(sys.argv) > 2 else "src-tauri/target/smoke-linux").resolve()
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="mulsanne-smoke-") as data, (output / "app.log").open("w") as log:
        env = os.environ.copy()
        env.pop("WAYLAND_DISPLAY", None)
        env.pop("GDK_BACKEND", None)
        env.pop("WEBKIT_DISABLE_DMABUF_RENDERER", None)
        env.update(LANG="C.UTF-8", LC_ALL="C.UTF-8", LANGUAGE="C", XDG_DATA_HOME=data + "/data", XDG_CONFIG_HOME=data + "/config",
                   APPIMAGE_EXTRACT_AND_RUN="1")
        # WebKit refuses to start as root in disposable Docker build containers.
        if os.geteuid() == 0:
            env["WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS"] = "1"
        processes = []
        try:
            # Hyprland reserves Xn_ as well as Xn. Avoid automatic allocation
            # starting at :0, which can collide with a desktop's reserved socket.
            while True:
                number = 100 + secrets.randbelow(10000)
                if not any(Path(path).exists() for path in [
                    f"/tmp/.X{number}-lock", f"/tmp/.X11-unix/X{number}", f"/tmp/.X11-unix/X{number}_",
                ]):
                    break
            env["DISPLAY"] = f":{number}"
            server = subprocess.Popen(["Xvfb", env["DISPLAY"], "-nolisten", "tcp",
                                       "-screen", "0", "1600x1000x24"], stdout=log, stderr=log)
            processes.append(server)
            for _ in range(100):
                assert server.poll() is None, "private X11 server failed to start"
                ready = subprocess.run(["xdotool", "getdisplaygeometry"], env=env, capture_output=True)
                if ready.returncode == 0:
                    break
                time.sleep(0.1)
            else:
                raise RuntimeError("private X11 server did not become ready")
            for command in [["openbox"], ["xcompmgr"]]:
                processes.append(subprocess.Popen(command, env=env, stdout=log, stderr=log))
            time.sleep(1)
            app = subprocess.Popen([str(binary)], env=env, stdout=log, stderr=log)
            processes.append(app)

            def run(*args):
                return subprocess.check_output(args, env=env, text=True, stderr=subprocess.STDOUT).strip()

            def wait_for(condition):
                deadline = time.monotonic() + 20
                while not condition():
                    assert app.poll() is None, f"app exited with {app.returncode}; see {output / 'app.log'}"
                    assert time.monotonic() < deadline, "UI smoke test timed out"
                    time.sleep(0.1)

            def window(title):
                result = subprocess.run(["xdotool", "search", "--name", "^" + title + "$"],
                                        env=env, text=True, capture_output=True)
                return result.stdout.splitlines()[-1] if result.returncode == 0 else None

            wait_for(lambda: window("Mulsanne") and window("Mulsanne HUD"))
            overlay = window("Mulsanne HUD")
            assert "_NET_WM_WINDOW_TYPE_DOCK" in run("xprop", "-id", overlay, "_NET_WM_WINDOW_TYPE")
            time.sleep(3)

            # Query the X11 input shape, not just whether a window is visible.
            x11 = ctypes.CDLL(ctypes.util.find_library("X11"))
            ext = ctypes.CDLL(ctypes.util.find_library("Xext"))
            x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
            x11.XOpenDisplay.restype = ctypes.c_void_p
            connection = x11.XOpenDisplay(env["DISPLAY"].encode())
            assert connection
            x11.XFree.argtypes = [ctypes.c_void_p]
            x11.XCloseDisplay.argtypes = [ctypes.c_void_p]

            class Rectangle(ctypes.Structure):
                _fields_ = [("x", ctypes.c_short), ("y", ctypes.c_short),
                            ("width", ctypes.c_ushort), ("height", ctypes.c_ushort)]

            ext.XShapeGetRectangles.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int,
                                               ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_int)]
            ext.XShapeGetRectangles.restype = ctypes.POINTER(Rectangle)

            def input_area():
                count, ordering = ctypes.c_int(), ctypes.c_int()
                rectangles = ext.XShapeGetRectangles(connection, int(overlay), 2,
                                                     ctypes.byref(count), ctypes.byref(ordering))
                area = sum(rectangles[i].width * rectangles[i].height for i in range(count.value))
                x11.XFree(rectangles)
                return area

            try:
                wait_for(lambda: input_area() <= 1)
                run("import", "-window", "root", str(output / "initial.png"))
                run("xdotool", "key", "ctrl+shift+F9")
                wait_for(lambda: input_area() > 1000)
                time.sleep(1)
                run("import", "-window", "root", str(output / "edit.png"))
                # The edit UI must actually render, beyond just changing OS input state.
                diff = subprocess.run(["compare", "-metric", "AE", str(output / "initial.png"),
                                       str(output / "edit.png"), "null:"], capture_output=True, text=True)
                assert diff.returncode in [0, 1], diff.stderr
                assert float(diff.stderr.split()[0]) > 1000, "edit UI did not render"
                run("xdotool", "key", "ctrl+shift+F9")
                wait_for(lambda: input_area() <= 1)
                run("xdotool", "key", "ctrl+shift+F10")
                wait_for(lambda: "IsUnMapped" in run("xwininfo", "-id", overlay, "-stats"))
                run("xdotool", "key", "ctrl+shift+F10")
                wait_for(lambda: "IsViewable" in run("xwininfo", "-id", overlay, "-stats"))
                assert list(Path(data).rglob("laps.db")), "lap database was not created"
                assert app.poll() is None
                print("PASS: startup, rendering, click-through, edit mode, global shortcuts, visibility, data storage")
                print(f"Screenshots and logs: {output}")
            finally:
                x11.XCloseDisplay(connection)
        finally:
            for process in reversed(processes):
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


if __name__ == "__main__":
    main()
