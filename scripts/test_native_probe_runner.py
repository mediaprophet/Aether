"""Exercise the Windows probe watchdog with real, workspace-owned child processes."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
PWSH = shutil.which("pwsh")
RUSTC = shutil.which("rustc")


@unittest.skipUnless(os.name == "nt" and PWSH and RUSTC, "requires Windows, PowerShell 7 and Rust")
class NativeProbeRunnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        target = ROOT / "target"
        target.mkdir(exist_ok=True)
        # Retain these small receipts under ignored target for failure diagnosis.
        cls.directory = Path(tempfile.mkdtemp(prefix="native-probe-runner-", dir=target))
        cls.executable = cls.directory / "probe.exe"
        source = cls.directory / "probe.rs"
        source.write_text('''
fn main() {
    println!("probe started");
    eprintln!("fixture diagnostic");
    if std::env::args().nth(1).as_deref() == Some("sleep") {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
    println!("probe completed");
}
''', encoding="utf-8")
        subprocess.run([RUSTC, str(source), "-o", str(cls.executable)],
                       check=True, capture_output=True, text=True, timeout=60)

    def run_probe(self, mode, seconds):
        prefix = self.directory / mode
        result = subprocess.run([
            PWSH, "-NoLogo", "-NoProfile", "-File", str(ROOT / "scripts/run_native_probe.ps1"),
            "-Executable", str(self.executable), "-LogPrefix", str(prefix),
            "-ProbeProfile", "watchdog-contract-test", "-TimeoutSeconds", str(seconds),
            "-ProgramArguments", mode,
        ], capture_output=True, text=True, timeout=20)
        receipt_path = Path(str(prefix) + ".json")
        self.assertTrue(receipt_path.is_file(), result.stdout + result.stderr)
        return result, json.loads(receipt_path.read_text(encoding="utf-8"))

    def test_completed_probe_records_both_streams_and_executable_hash(self):
        result, receipt = self.run_probe("complete", 3)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("probe completed", result.stdout)
        self.assertIn("fixture diagnostic", result.stderr)
        self.assertFalse(receipt["timed_out"])
        self.assertTrue(receipt["capture_complete"] and receipt["process_exited"])
        self.assertEqual(receipt["program_exit_code"], 0)
        self.assertEqual(len(receipt["executable_sha256"]), 64)
        self.assertNotIn("arguments", receipt)

    def test_timeout_stops_the_owned_probe_and_records_failure(self):
        result, receipt = self.run_probe("sleep", 1)
        self.assertEqual(result.returncode, 124, result.stdout + result.stderr)
        self.assertTrue(receipt["timed_out"] and receipt["process_exited"])
        self.assertTrue(receipt["capture_complete"])
        self.assertNotEqual(receipt["program_exit_code"], 0)
        self.assertNotIn("probe completed", result.stdout)


if __name__ == "__main__":
    unittest.main()
