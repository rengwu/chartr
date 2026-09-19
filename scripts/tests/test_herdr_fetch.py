"""Exercise sidecar reuse/failure handling without downloading or compiling Herdr."""
import io
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class FetchTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        for name in ('vendor/herdr/fetch.sh', 'crates/chartr-herdr/src/lib.rs',
                     'rust-toolchain.toml'):
            dest = self.root / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, dest)
        self.target = ('aarch64' if platform.machine() in ('arm64', 'aarch64') else 'x86_64')
        self.target += '-apple-darwin' if platform.system() == 'Darwin' else '-unknown-linux-gnu'
        pins = (self.root / 'crates/chartr-herdr/src/lib.rs').read_text()
        self.toolchain = (self.root / 'rust-toolchain.toml').read_text().split('channel = "')[1].split('"')[0]
        self.version = pins.split('SUPPORTED_HERDR_VERSION: &str = "')[1].split('"')[0]
        self.upstream_version = pins.split('SUPPORTED_HERDR_UPSTREAM_VERSION: &str = "')[1].split('"')[0]
        self.revision = pins.split('SUPPORTED_HERDR_REVISION: &str = "')[1].split('"')[0]
        self.binary = self.root / 'vendor/herdr' / self.target / 'herdr'
        self.bin = self.root / 'fake-bin'
        self.bin.mkdir()
        self.env = dict(os.environ, PATH=f'{self.bin}:{os.environ["PATH"]}',
                        ZIG=str(self.bin / 'zig'), TEST_ROOT=str(self.root),
                        TEST_VERSION=self.version)
        self.env.pop('HERDR_REBUILD', None)
        self.script('codesign', 'exit 0')
        self.script('zig', 'echo 0.15.2')
        self.script('cargo', '''
echo "$*" >> "$TEST_ROOT/builds"
[ "${FAIL_BUILD:-0}" = 0 ] || exit 42
while [ "$1" != --target ]; do shift; done
shift
mkdir -p "$CARGO_TARGET_DIR/$1/release"
printf '#!/bin/sh\necho "herdr %s"\n' "$TEST_VERSION" > "$CARGO_TARGET_DIR/$1/release/herdr"
chmod +x "$CARGO_TARGET_DIR/$1/release/herdr"
''')
        with tarfile.open(self.root / 'fixture.tar.gz', 'w:gz') as archive:
            for name, data in [('Cargo.toml', f'version = "{self.upstream_version}"\n'.encode()),
                               ('LICENSE', b'test license\n')]:
                info = tarfile.TarInfo('herdr/' + name)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))
        self.script('curl', '''
echo download >> "$TEST_ROOT/downloads"
while [ "$1" != -o ]; do shift; done
cp "$TEST_ROOT/fixture.tar.gz" "$2"
''')

    def script(self, name, contents):
        path = self.bin / name
        path.write_text('#!/bin/sh\nset -eu\n' + contents + '\n')
        path.chmod(0o755)

    def seed_binary(self, version):
        self.binary.parent.mkdir(parents=True, exist_ok=True)
        self.binary.write_text(f'#!/bin/sh\necho "herdr {version}"\n')
        self.binary.chmod(0o755)

    def run_fetch(self, *args, **env):
        return subprocess.run(['sh', 'vendor/herdr/fetch.sh', *args], cwd=self.root,
                              env=dict(self.env, **env), text=True, capture_output=True)

    def test_matching_binary_needs_no_zig_or_network(self):
        self.seed_binary(self.version)
        result = self.run_fetch(ZIG='/missing-zig')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('reusing', result.stdout)
        self.assertFalse((self.root / 'downloads').exists())
        self.assertFalse((self.root / 'builds').exists())

    def test_cold_build_then_reuse_and_forced_rebuild(self):
        result = self.run_fetch()
        self.assertEqual(result.returncode, 0, result.stderr)
        result = self.run_fetch()
        self.assertEqual(result.returncode, 0, result.stderr)
        result = self.run_fetch(HERDR_REBUILD='1')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / 'downloads').read_text().splitlines(), ['download'])
        builds = (self.root / 'builds').read_text().splitlines()
        self.assertEqual(len(builds), 2)
        for build in builds:
            self.assertTrue(build.startswith(f'+{self.toolchain} build --release --locked'))
            self.assertIn('--timings', build)
        self.assertTrue((self.root / 'vendor/herdr/.build/source' / self.revision).is_dir())
        self.assertTrue((self.root / 'vendor/herdr/.build/target' / self.target / 'release/herdr').exists())

    def test_stale_sidecar_is_replaced(self):
        self.seed_binary('old')
        result = self.run_fetch()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(self.version, self.binary.read_text())

    def test_failed_build_preserves_previous_binary(self):
        self.seed_binary('old')
        result = self.run_fetch(FAIL_BUILD='1')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('old', self.binary.read_text())
        self.assertTrue((self.root / 'vendor/herdr/.build/source' / self.revision).is_dir())

    def test_wrong_built_version_is_not_published(self):
        self.seed_binary('old')
        result = self.run_fetch(TEST_VERSION='incorrect')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('old', self.binary.read_text())

    def test_invalid_target_fails_before_network(self):
        result = self.run_fetch('x86_64-unknown-linuz-gnu')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / 'downloads').exists())

    def test_wrong_zig_fails_before_network(self):
        self.script('zig', 'echo 0.16.0')
        result = self.run_fetch()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('requires Zig 0.15.2', result.stderr)
        self.assertFalse((self.root / 'downloads').exists())


if __name__ == '__main__':
    unittest.main()
