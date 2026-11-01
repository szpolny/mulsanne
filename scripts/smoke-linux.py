#!/usr/bin/env python3
"""Exercise the packaged UI on a private X11 display; never touch user data.

Needs Xvfb, Openbox, xcompmgr, xdotool, xwininfo, xprop, ImageMagick, and libX11/libXext.
Usage: python3 scripts/smoke-linux.py [binary] [output-directory]
"""
import ctypes
import ctypes.util
import os
import re
import json
import struct
import threading
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
        # Exercise the non-composited path even on GL-capable Xvfb setups.
        env["WEBKIT_DISABLE_DMABUF_RENDERER"] = "1"
        env.update(LANG="C.UTF-8", LC_ALL="C.UTF-8", LANGUAGE="C", XDG_DATA_HOME=data + "/data", XDG_CONFIG_HOME=data + "/config",
                   APPIMAGE_EXTRACT_AND_RUN="1")
        # WebKit refuses to start as root in disposable Docker build containers.
        if os.geteuid() == 0:
            env["WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS"] = "1"
        # Drive changing session data without a real game or the user's mirror.
        mirror = Path(data) / "LMU_Data"
        env["MULSANNE_LMU_SHM"] = str(mirror)
        config_dir = Path(data) / "config/xyz.mulsanne"
        config_dir.mkdir(parents=True)
        widgets = {name: {"enabled": name == "session", "x": 20, "y": 20} for name in
                   ["dashboard", "inputs", "delta", "laptiming", "standings", "relative", "fuel",
                    "tyres", "session", "flags", "trackmap", "weather", "damage", "electronics"]}
        (config_dir / "config.json").write_text(json.dumps({"general": {"onlyOnTrack": False}, "widgets": widgets}))
        stopped = threading.Event()

        def telemetry():
            frame = bytearray(324825)
            struct.pack_into("<i", frame, 1632 + 64, 1)  # practice
            struct.pack_into("<d", frame, 1632 + 76, 3600.0)  # session end
            frame[128464] = frame[128466] = 1  # one player telemetry entry
            start = time.monotonic()
            while not stopped.is_set():
                elapsed = time.monotonic() - start
                struct.pack_into("<d", frame, 1632 + 68, elapsed)
                struct.pack_into("<d", frame, 128468 + 12, elapsed)
                pending = mirror.with_suffix(".next")
                pending.write_bytes(frame)
                pending.replace(mirror)
                stopped.wait(1 / 30)

        writer = threading.Thread(target=telemetry)
        writer.start()
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

            # A fake LMU X11 client verifies both initial bounds and later resizes.
            x11.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
            x11.XDefaultRootWindow.restype = ctypes.c_ulong
            x11.XCreateSimpleWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int,
                ctypes.c_int, ctypes.c_uint, ctypes.c_uint, ctypes.c_uint, ctypes.c_ulong, ctypes.c_ulong]
            x11.XCreateSimpleWindow.restype = ctypes.c_ulong
            fake = x11.XCreateSimpleWindow(connection, x11.XDefaultRootWindow(connection), 80, 80, 1200, 900, 0, 0, 0)
            class ClassHint(ctypes.Structure):
                _fields_ = [("name", ctypes.c_char_p), ("class_name", ctypes.c_char_p)]
            hint = ClassHint(b"steam_app_2399420", b"steam_app_2399420")
            x11.XSetClassHint.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(ClassHint)]
            x11.XSetClassHint(connection, fake, ctypes.byref(hint))
            x11.XMapWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
            x11.XMapWindow(connection, fake)
            x11.XFlush.argtypes = [ctypes.c_void_p]
            x11.XFlush(connection)

            def geometry(window_id):
                # xdotool double-counts parent offsets for decorated clients.
                info = run("xwininfo", "-id", str(window_id), "-stats")
                fields = {"X": "Absolute upper-left X", "Y": "Absolute upper-left Y",
                          "WIDTH": "Width", "HEIGHT": "Height"}
                return {key: re.search(r"^\s*" + label + r":\s*(-?\d+)", info, re.MULTILINE).group(1)
                        for key, label in fields.items()}

            def same_bounds():
                game, hud = geometry(fake), geometry(overlay)
                return all(game[key] == hud[key] for key in ["X", "Y", "WIDTH", "HEIGHT"])

            try:
                wait_for(same_bounds)
                x11.XMoveResizeWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int,
                                                 ctypes.c_int, ctypes.c_uint, ctypes.c_uint]
                x11.XMoveResizeWindow(connection, fake, 140, 110, 1000, 800)
                x11.XFlush(connection)
                wait_for(same_bounds)
                wait_for(lambda: input_area() <= 1)
                # Sample static labels while the countdown changes. A WebKit
                # snapshot alone misses the transparent backing-store regression.
                hud = geometry(overlay)
                crop = f"260x18+{int(hud['X']) + 24}+{int(hud['Y']) + 24}"
                for sample in range(4):
                    time.sleep(0.3)
                    shot = output / f"live-{sample}.png"
                    run("import", "-window", "root", str(shot))
                    pixels = float(run("convert", str(shot), "-crop", crop, "-colorspace", "Gray",
                                       "-threshold", "40%", "-format", "%[fx:mean*w*h]", "info:").splitlines()[-1])
                    assert pixels > 30, f"static HUD labels disappeared during telemetry updates: {pixels} pixels"
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
                print("PASS: startup, live static rendering, game bounds and resize, click-through, edit mode, global shortcuts, visibility, data storage")
                print(f"Screenshots and logs: {output}")
            finally:
                x11.XCloseDisplay(connection)
        finally:
            stopped.set()
            writer.join()
            for process in reversed(processes):
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


if __name__ == "__main__":
    main()
