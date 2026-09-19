"""Exercise real makepkg packages and Git pushes to an isolated local AUR stand-in."""
import hashlib
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
ARCH_TOOLS = all(shutil.which(tool) for tool in ('makepkg', 'vercmp', 'cc', 'bsdtar'))


@unittest.skipUnless(ARCH_TOOLS and os.getuid() != 0 and platform.machine() == 'x86_64',
                     'requires Arch tools on x86_64 and an unprivileged user')
class AurPackageTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.work = Path(self.tmp.name)
        self.env = dict(os.environ, PKGEXT='.pkg.tar.zst')

    def run_command(self, args, cwd=None):
        return subprocess.run(args, cwd=cwd, env=self.env, text=True, capture_output=True)

    def checked(self, args, cwd=None):
        result = self.run_command(args, cwd)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result.stdout

    def recipe(self, version):
        bundle = self.work / f'chartr-{version}-linux-x86_64'
        binaries = bundle / 'usr/lib/chartr'
        binaries.mkdir(parents=True)
        source = self.work / 'fixture.c'
        source.write_text('#include <stdio.h>\nint main(void) { puts("fixture"); return 0; }\n')
        self.checked(['cc', str(source), '-o', str(binaries / 'chartr')])
        shutil.copy2(binaries / 'chartr', binaries / 'herdr')
        (bundle / 'usr/bin').mkdir()
        (bundle / 'usr/bin/chartr').symlink_to('../lib/chartr/chartr')
        desktop = bundle / 'usr/share/applications/chartr.desktop'
        desktop.parent.mkdir(parents=True)
        shutil.copyfile(ROOT / 'packaging/linux/chartr.desktop', desktop)
        (bundle / 'BUILD-INFO.txt').write_text(f'chartr: {version}\n')
        archive = bundle.with_name(bundle.name + '.tar.gz')
        with tarfile.open(archive, 'w:gz') as tar:
            tar.add(bundle, arcname=bundle.name)
        recipe = self.work / f'recipe-{version}'
        self.checked(['bash', str(ROOT / 'scripts/package-aur.sh'), str(archive), str(recipe)])
        return recipe, archive

    def test_stable_and_candidate_build_from_the_same_aur_recipe(self):
        for version, pkgver in [('0.3.0', '0.3.0'), ('0.3.0-rc.1', '0.3.0rc1')]:
            with self.subTest(version=version):
                recipe, archive = self.recipe(version)
                metadata = (recipe / '.SRCINFO').read_text()
                self.assertIn(f'pkgver = {pkgver}\n', metadata)
                self.assertIn(f'provides = chartr={pkgver}\n', metadata)
                self.assertIn('conflicts = chartr\n', metadata)
                self.assertIn(f'https://github.com/rengwu/chartr/releases/download/v{version}/{archive.name}', metadata)
                self.assertIn(hashlib.sha256(archive.read_bytes()).hexdigest(), metadata)
                self.assertNotIn('SKIP', metadata)
                shutil.copyfile(archive, recipe / archive.name)
                self.checked(['makepkg', '--noconfirm', '--nosign'], recipe)
                package, = recipe.glob('*.pkg.tar.zst')
                self.assertEqual(package.name, f'chartr-bin-{pkgver}-1-x86_64.pkg.tar.zst')
                extracted = recipe / 'installed'
                extracted.mkdir()
                self.checked(['bsdtar', '-xf', str(package), '-C', str(extracted)])
                self.assertEqual(os.readlink(extracted / 'usr/bin/chartr'), '../lib/chartr/chartr')
                self.checked([str(extracted / 'usr/bin/chartr')])
                for binary in ('chartr', 'herdr'):
                    self.assertEqual((extracted / 'usr/lib/chartr' / binary).read_bytes(),
                                     (self.work / archive.name[:-7] / 'usr/lib/chartr' / binary).read_bytes())
                self.assertTrue((extracted / 'usr/share/applications/chartr.desktop').is_file())
                self.assertEqual({p.name for p in (extracted / 'usr/lib/chartr').iterdir()}, {'chartr', 'herdr'})
                self.assertIn(version, (extracted / 'usr/share/doc/chartr/BUILD-INFO.txt').read_text())
        self.assertEqual(self.checked(['vercmp', '0.3.0rc1', '0.3.0']).strip(), '-1')

    def test_tampered_source_fails_checksum_validation(self):
        recipe, archive = self.recipe('0.3.0')
        (recipe / archive.name).write_bytes(archive.read_bytes() + b'tampered')
        result = self.run_command(['makepkg', '--verifysource'], recipe)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('validity check', result.stderr)

    def test_wrong_architecture_or_version_is_rejected(self):
        for name in ('chartr-0.3.0-linux-aarch64.tar.gz', 'chartr-main-linux-x86_64.tar.gz'):
            archive = self.work / name
            archive.touch()
            result = self.run_command(['bash', str(ROOT / 'scripts/package-aur.sh'),
                                       str(archive), str(self.work / 'recipe')])
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((self.work / 'recipe').exists())

    def aur_checkout(self):
        remote = self.work / 'aur.git'
        checkout = self.work / 'aur'
        self.checked(['git', 'init', '--bare', '--initial-branch=master', str(remote)])
        self.checked(['git', 'clone', str(remote), str(checkout)])
        return remote, checkout

    def publish(self, recipe, checkout):
        return self.run_command(['bash', str(ROOT / 'scripts/publish-aur.sh'),
                                 str(recipe), str(checkout)])

    def test_first_publication_update_and_retry(self):
        remote, checkout = self.aur_checkout()
        recipe, _ = self.recipe('0.3.0')
        result = self.publish(recipe, checkout)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        first = self.checked(['git', '--git-dir', str(remote), 'rev-parse', 'master'])
        result = self.publish(recipe, checkout)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('already contains', result.stdout)
        self.assertEqual(first, self.checked(['git', '--git-dir', str(remote), 'rev-parse', 'master']))
        newer, _ = self.recipe('0.3.1')
        result = self.publish(newer, checkout)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.checked(['git', '--git-dir', str(remote), 'rev-list', '--count', 'master']).strip(), '2')
        tracked = self.checked(['git', '--git-dir', str(remote), 'ls-tree', '--name-only', 'master'])
        self.assertEqual(set(tracked.splitlines()), {'PKGBUILD', '.SRCINFO'})

    def test_older_release_and_changed_same_version_cannot_overwrite_aur(self):
        remote, checkout = self.aur_checkout()
        recipe, _ = self.recipe('0.3.1')
        self.assertEqual(self.publish(recipe, checkout).returncode, 0)
        first = self.checked(['git', '--git-dir', str(remote), 'rev-parse', 'master'])
        older, _ = self.recipe('0.3.0')
        for candidate in (older, recipe):
            with (candidate / 'PKGBUILD').open('a') as file:
                file.write('# changed\n')
            result = self.publish(candidate, checkout)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('refusing to overwrite', result.stderr)
            self.assertEqual(first, self.checked(['git', '--git-dir', str(remote), 'rev-parse', 'master']))

    def test_candidate_cannot_be_published_to_stable_aur(self):
        _, checkout = self.aur_checkout()
        recipe, _ = self.recipe('0.3.0-rc.1')
        result = self.publish(recipe, checkout)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('only stable versions', result.stderr)
        self.assertFalse((checkout / 'PKGBUILD').exists())


if __name__ == '__main__':
    unittest.main()
