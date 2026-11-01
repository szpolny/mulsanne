import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("bridge_launcher", Path(__file__).parents[1] / "scripts/run-lmu-bridge.py")
launcher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(launcher)


class BridgeLauncherTests(unittest.TestCase):
    def test_finds_game_not_a_steam_launcher_mentioning_the_game(self):
        with tempfile.TemporaryDirectory() as directory:
            proc = Path(directory)
            for pid, command in [(10, b"steam\0Le Mans Ultimate.exe\0"),
                                 (20, b"Z:\\Steam Library\\Le Mans Ultimate.exe\0")]:
                entry = proc / str(pid)
                entry.mkdir()
                (entry / "cmdline").write_bytes(command)
                (entry / "environ").write_bytes(b"WINEPREFIX=/prefix\0WINELOADER=/proton/wine64\0")
            pid, environment = launcher.find_game(proc)
            self.assertEqual(pid, 20)
            self.assertEqual(environment["WINELOADER"], "/proton/wine64")

    def test_reports_game_not_running(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(RuntimeError, "Start Le Mans Ultimate"):
                launcher.find_game(Path(directory))

    def test_enters_runtime_before_setting_library_paths_and_uses_game_loader(self):
        environment = {"WINEPREFIX": "/Steam Library/prefix", "WINELOADER": "/Proton/wine64",
                       "LD_LIBRARY_PATH": "/runtime/libs", "PRIVATE_TOKEN": "do-not-copy"}
        with patch.object(launcher.shutil, "which", return_value="/bin/nsenter"):
            command, child_env = launcher.launch_command(123, environment, Path("/bridge path.exe"), [], ["--user", "--mount"])
        self.assertEqual(command[0], "/bin/nsenter")
        self.assertIn("--preserve-credentials", command)
        self.assertGreater(command.index("LD_LIBRARY_PATH=/runtime/libs"), command.index("/usr/bin/env"))
        self.assertEqual(command[-2:], ["/Proton/wine64", "/bridge path.exe"])
        self.assertNotIn("PRIVATE_TOKEN=do-not-copy", command)
        self.assertNotEqual(child_env.get("LD_LIBRARY_PATH"), "/runtime/libs")

    def test_missing_namespace_tool_has_an_actionable_error(self):
        environment = {"WINEPREFIX": "/prefix", "WINELOADER": "/Proton/wine64"}
        with patch.object(launcher.shutil, "which", return_value=None):
            with self.assertRaisesRegex(RuntimeError, "install util-linux"):
                launcher.launch_command(123, environment, Path("/bridge.exe"), [], ["--mount"])

    def test_same_runtime_needs_no_namespace_tool(self):
        environment = {"WINEPREFIX": "/prefix", "WINELOADER": "/Proton/wine64"}
        command, child_env = launcher.launch_command(123, environment, Path("/bridge.exe"), ["Z:\\mirror"], [])
        self.assertEqual(command, ["/Proton/wine64", "/bridge.exe", "Z:\\mirror"])
        self.assertEqual(child_env["WINEPREFIX"], "/prefix")


if __name__ == "__main__":
    unittest.main()
