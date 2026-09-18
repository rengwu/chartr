"""Package tiny native ELF fixtures using the actual Debian packaging tools."""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(shutil.which('dpkg-shlibdeps') and shutil.which('cc'),
                     'requires Ubuntu/Debian packaging tools and a C compiler')
class PackageTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.work = Path(self.tmp.name)
        self.binaries = self.work / 'binaries'
        self.binaries.mkdir()
        self.output = self.work / 'packages'
        pins = (ROOT / 'crates/chartr-herdr/src/lib.rs').read_text()
        self.herdr_version = pins.split('SUPPORTED_HERDR_VERSION: &str = "')[1].split('"')[0]
        self.compile('chartr', 'int main(void) { return 0; }')
        self.compile('herdr', '#include <stdio.h>\nint main(void) { puts("herdr '
                     + self.herdr_version + '"); return 0; }')
        # Packaging records Rust metadata but does not require a Rust compiler.
        tools = self.work / 'tools'
        tools.mkdir()
        rustc = tools / 'rustc'
        rustc.write_text('#!/bin/sh\necho "rustc package-test-fixture"\n')
        rustc.chmod(0o755)
        self.env = dict(os.environ, PATH=f'{tools}:{os.environ["PATH"]}')

    def compile(self, name, source):
        c = self.work / (name + '.c')
        c.write_text(source)
        subprocess.run(['cc', str(c), '-o', str(self.binaries / name)], check=True)

    def package(self):
        return subprocess.run(['bash', str(ROOT / 'scripts/package-linux.sh'),
                               str(self.binaries), str(self.output)], env=self.env,
                              text=True, capture_output=True)

    def test_both_formats_preserve_sidecar_layout_and_binary_contents(self):
        before = {p.name: p.read_bytes() for p in self.binaries.iterdir()}
        result = self.package()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        archive, = self.output.glob('*.tar.gz')
        deb, = self.output.glob('*.deb')
        unpacked = self.work / 'archive'
        with tarfile.open(archive) as tar:
            tar.extractall(unpacked, filter='data')
        bundle, = unpacked.iterdir()
        self.assertEqual(os.readlink(bundle / 'usr/bin/chartr'), '../lib/chartr/chartr')
        subprocess.run([str(bundle / 'usr/bin/chartr')], check=True)
        deb_root = self.work / 'deb'
        subprocess.run(['dpkg-deb', '-x', str(deb), str(deb_root)], check=True)
        for binary in ('chartr', 'herdr'):
            self.assertEqual((bundle / 'usr/lib/chartr' / binary).read_bytes(),
                             (deb_root / 'usr/lib/chartr' / binary).read_bytes())
            self.assertEqual(before[binary], (self.binaries / binary).read_bytes())
        deps = subprocess.check_output(['dpkg-deb', '-f', str(deb), 'Depends'], text=True)
        self.assertIn('libc6', deps)
        self.assertIn('libvulkan1', deps)
        checksums, = self.output.glob('SHA256SUMS-*')
        for line in checksums.read_text().splitlines():
            digest, name = line.split()
            self.assertEqual(digest, hashlib.sha256((self.output / name).read_bytes()).hexdigest())

    def test_rejects_wrong_sidecar_before_creating_packages(self):
        self.compile('herdr', '#include <stdio.h>\nint main(void) { puts("herdr wrong"); }')
        result = self.package()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('does not match', result.stderr)
        self.assertEqual(list(self.output.iterdir()), [])

    def test_rejects_missing_sidecar(self):
        (self.binaries / 'herdr').unlink()
        result = self.package()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('missing chartr or herdr', result.stderr)


if __name__ == '__main__':
    unittest.main()
