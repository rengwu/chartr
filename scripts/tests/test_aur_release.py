"""Validate release artifacts before they can be sent to AUR; no network needed."""
import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('aur_release', ROOT / 'scripts/prepare-aur-release.py')
AUR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUR)


class AurReleaseTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.work = Path(self.tmp.name)
        self.bundle = self.work / 'chartr-0.3.0-aur.tar.gz'
        self.manifest = self.work / 'SHA256SUMS-x86_64'
        self.output = self.work / 'recipe'
        self.metadata = f'''pkgbase = chartr-bin
    pkgver = 0.3.0
    pkgrel = 1
    arch = x86_64
    source = https://github.com/rengwu/chartr/releases/download/v0.3.0/chartr-0.3.0-linux-x86_64.tar.gz
    sha256sums = {'a' * 64}
    provides = chartr=0.3.0
    conflicts = chartr
pkgname = chartr-bin
'''

    def artifacts(self, metadata=None, extra=None):
        files = {'PKGBUILD': b'# tested recipe\n',
                 '.SRCINFO': (metadata or self.metadata).encode()}
        if extra:
            files.update(extra)
        with tarfile.open(self.bundle, 'w:gz') as tar:
            for name, data in files.items():
                member = tarfile.TarInfo(name)
                member.size = len(data)
                tar.addfile(member, io.BytesIO(data))
        digest = hashlib.sha256(self.bundle.read_bytes()).hexdigest()
        self.manifest.write_text(f'{digest}  {self.bundle.name}\n'
                                 f'{"a" * 64}  chartr-0.3.0-linux-x86_64.tar.gz\n')

    def prepare(self):
        AUR.prepare_recipe('v0.3.0', self.bundle, self.manifest, self.output)

    def test_published_stable_release_is_accepted(self):
        AUR.validate_release('v0.3.0', {'tagName': 'v0.3.0', 'isDraft': False, 'isPrerelease': False})
        self.artifacts()
        self.prepare()
        self.assertEqual({p.name for p in self.output.iterdir()}, {'PKGBUILD', '.SRCINFO'})
        self.assertEqual((self.output / '.SRCINFO').read_text(), self.metadata)

    def test_drafts_prereleases_and_mismatched_tags_are_rejected(self):
        for tag, release in [
            ('v0.3.0-rc.1', {}), ('main', {}),
            ('v0.3.0', {'tagName': 'v0.3.0', 'isDraft': True, 'isPrerelease': False}),
            ('v0.3.0', {'tagName': 'v0.3.0', 'isDraft': False, 'isPrerelease': True}),
            ('v0.3.0', {'tagName': 'v0.3.1', 'isDraft': False, 'isPrerelease': False}),
        ]:
            with self.subTest(tag=tag, release=release), self.assertRaises(ValueError):
                AUR.validate_release(tag, release)

    def test_modified_recipe_archive_is_rejected(self):
        self.artifacts()
        self.bundle.write_bytes(self.bundle.read_bytes() + b'tampered')
        with self.assertRaisesRegex(ValueError, 'checksum'):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_metadata_must_match_the_published_binary(self):
        for before, after in [('0.3.0', '0.3.1'), ('a' * 64, 'SKIP'),
                              ('chartr-bin', 'other-bin'),
                              ('https://github.com/', 'https://example.com/')]:
            with self.subTest(after=after):
                self.artifacts(self.metadata.replace(before, after))
                with self.assertRaisesRegex(ValueError, 'metadata'):
                    self.prepare()
                self.assertFalse(self.output.exists())

    def test_bundle_cannot_write_unexpected_paths(self):
        self.artifacts(extra={'../outside': b'not allowed'})
        with self.assertRaisesRegex(ValueError, 'only PKGBUILD'):
            self.prepare()
        self.assertFalse(self.output.exists())
        self.assertFalse((self.work / 'outside').exists())

    def test_duplicate_checksum_entry_is_rejected(self):
        self.artifacts()
        self.manifest.write_text(self.manifest.read_text() * 2)
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            self.prepare()


if __name__ == '__main__':
    unittest.main()
